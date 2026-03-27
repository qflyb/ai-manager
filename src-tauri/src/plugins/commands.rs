use crate::plugins::command_targets::collect_tool_commands_dirs;
use crate::plugins::github;
use crate::plugins::models::*;
use crate::plugins::storage;
use crate::skills::commands as skill_commands;
use crate::skills::elevation::{
    execute_with_optional_elevation, ElevatedSymlinkAction, InstallOperationError,
};
use crate::skills::fs_utils;
use crate::skills::parser;
use crate::skills::registry;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn read_plugin_json(plugin_dir: &Path) -> Result<PluginJson, String> {
    let plugin_json_path = plugin_dir.join(".claude-plugin").join("plugin.json");
    if !plugin_json_path.exists() {
        return Err(format!(
            "No .claude-plugin/plugin.json found at {}",
            plugin_dir.display()
        ));
    }

    let content = fs::read_to_string(&plugin_json_path)
        .map_err(|e| format!("Failed to read plugin.json: {}", e))?;

    serde_json::from_str::<PluginJson>(&content)
        .map_err(|e| format!("Failed to parse plugin.json: {}", e))
}

pub(super) fn generate_plugin_id(source: &PluginSource, metadata: &PluginMetadata) -> String {
    match source {
        PluginSource::GitHub { owner, repo } => format!("{}--{}", owner, repo),
        PluginSource::Local { .. } => metadata
            .name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect(),
    }
}

pub(super) fn now_iso8601() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple ISO-like timestamp without chrono dependency
    format!("{}", now)
}

#[tauri::command]
pub fn add_plugin_local(path: String) -> Result<PluginEntry, String> {
    let plugin_dir = PathBuf::from(&path);
    if !plugin_dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !plugin_dir.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let plugin_json = read_plugin_json(&plugin_dir)?;
    let metadata: PluginMetadata = plugin_json.into();
    let source = PluginSource::Local { path: path.clone() };
    let id = generate_plugin_id(&source, &metadata);

    let mut reg = storage::load_registry()?;

    if reg.plugins.iter().any(|p| p.id == id) {
        return Err(format!("Plugin '{}' is already added", id));
    }

    let entry = PluginEntry {
        id,
        source,
        local_path: path,
        metadata,
        added_at: now_iso8601(),
        marketplace_id: None,
    };

    reg.plugins.push(entry.clone());
    storage::save_registry(&reg)?;

    Ok(entry)
}

#[tauri::command]
pub fn add_plugin_github(owner: String, repo: String) -> Result<PluginEntry, String> {
    let repos_dir = storage::get_repos_dir().ok_or("Cannot determine home directory")?;
    if !repos_dir.exists() {
        fs::create_dir_all(&repos_dir)
            .map_err(|e| format!("Failed to create repos directory: {}", e))?;
    }

    let target_dir = repos_dir.join(format!("{}--{}", owner, repo));

    if target_dir.exists() {
        // Already cloned, try to update
        github::pull_repo(&target_dir)?;
    } else {
        github::clone_repo(&owner, &repo, &target_dir)?;
    }

    let plugin_json = read_plugin_json(&target_dir).map_err(|e| {
        // Cleanup on failure
        let _ = fs::remove_dir_all(&target_dir);
        e
    })?;

    let metadata: PluginMetadata = plugin_json.into();
    let source = PluginSource::GitHub {
        owner: owner.clone(),
        repo: repo.clone(),
    };
    let id = generate_plugin_id(&source, &metadata);

    let mut reg = storage::load_registry()?;

    // Remove existing entry if re-adding
    reg.plugins.retain(|p| p.id != id);

    let entry = PluginEntry {
        id,
        source,
        local_path: target_dir.to_string_lossy().to_string(),
        metadata,
        added_at: now_iso8601(),
        marketplace_id: None,
    };

    reg.plugins.push(entry.clone());
    storage::save_registry(&reg)?;

    Ok(entry)
}

#[tauri::command]
pub fn add_plugin(input: String) -> Result<PluginEntry, String> {
    let trimmed = input.trim();

    // Check if it's a local filesystem path
    let path = PathBuf::from(trimmed);
    if path.is_absolute() && path.is_dir() {
        return add_plugin_local(trimmed.to_string());
    }

    // Treat as GitHub: parse owner/repo
    let stripped = trimmed
        .replace("https://github.com/", "")
        .replace("http://github.com/", "");
    let stripped = stripped.trim_end_matches(".git");
    let parts: Vec<&str> = stripped.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        return add_plugin_github(parts[0].to_string(), parts[1].to_string());
    }

    Err("Invalid input. Enter a local directory path or owner/repo".to_string())
}

#[tauri::command]
pub fn list_plugins() -> Result<Vec<PluginEntry>, String> {
    let reg = storage::load_registry()?;
    Ok(reg.plugins)
}

#[tauri::command]
pub fn remove_plugin(plugin_id: String) -> Result<(), String> {
    let mut reg = storage::load_registry()?;

    let entry = reg
        .plugins
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?
        .clone();

    // Delete cloned repo for GitHub plugins
    if let PluginSource::GitHub { .. } = &entry.source {
        let repo_path = PathBuf::from(&entry.local_path);
        if repo_path.exists() {
            fs::remove_dir_all(&repo_path)
                .map_err(|e| format!("Failed to delete cloned repo: {}", e))?;
        }
    }

    reg.plugins.retain(|p| p.id != plugin_id);
    storage::save_registry(&reg)?;

    Ok(())
}

#[tauri::command]
pub fn update_plugin(plugin_id: String) -> Result<PluginEntry, String> {
    let mut reg = storage::load_registry()?;

    let entry = reg
        .plugins
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?
        .clone();

    match &entry.source {
        PluginSource::GitHub { .. } => {
            let repo_path = PathBuf::from(&entry.local_path);
            github::pull_repo(&repo_path)?;

            let plugin_json = read_plugin_json(&repo_path)?;
            let metadata: PluginMetadata = plugin_json.into();

            // Update the entry in registry
            if let Some(existing) = reg.plugins.iter_mut().find(|p| p.id == plugin_id) {
                existing.metadata = metadata;
            }

            storage::save_registry(&reg)?;
            Ok(reg.plugins.into_iter().find(|p| p.id == plugin_id).unwrap())
        }
        PluginSource::Local { path } => {
            // Re-read plugin.json for local plugins
            let plugin_dir = PathBuf::from(path);
            let plugin_json = read_plugin_json(&plugin_dir)?;
            let metadata: PluginMetadata = plugin_json.into();

            if let Some(existing) = reg.plugins.iter_mut().find(|p| p.id == plugin_id) {
                existing.metadata = metadata;
            }

            storage::save_registry(&reg)?;
            Ok(reg.plugins.into_iter().find(|p| p.id == plugin_id).unwrap())
        }
    }
}

#[tauri::command]
pub fn list_plugin_contents(plugin_id: String) -> Result<PluginContents, String> {
    let reg = storage::load_registry()?;
    let entry = reg
        .plugins
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    let plugin_dir = PathBuf::from(&entry.local_path);
    let tool_registry = registry::get_tool_registry();

    // Collect tool skills and commands dirs for cross-referencing
    let tool_dirs: Vec<(String, Option<PathBuf>, Option<PathBuf>)> = tool_registry
        .iter()
        .filter_map(|t| {
            let config_dir = (t.dir_resolver)()?;
            if !config_dir.exists() {
                return None;
            }
            let skills_dir = t.def.skills_subdir.map(|s| config_dir.join(s));
            let commands_dir = t.def.commands_subdir.map(|c| config_dir.join(c));
            Some((t.def.id.to_string(), skills_dir, commands_dir))
        })
        .collect();

    // Scan skills
    let skills_dir = plugin_dir.join("skills");
    let mut skills = Vec::new();
    if skills_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if dir_name.starts_with('.') || dir_name.starts_with("__") {
                    continue;
                }

                let skill_file = path.join("SKILL.md");
                let (name, description) = if skill_file.exists() {
                    let content = fs::read_to_string(&skill_file).unwrap_or_default();
                    let parsed = parser::parse_skill_md(&content);
                    (
                        parsed
                            .frontmatter
                            .get("name")
                            .cloned()
                            .unwrap_or_else(|| dir_name.clone()),
                        parsed
                            .frontmatter
                            .get("description")
                            .cloned()
                            .unwrap_or_default(),
                    )
                } else {
                    (dir_name.clone(), String::new())
                };

                let skill_canonical = fs::canonicalize(&path).ok();
                let mut installed_in = Vec::new();

                for (tool_id, skills_dir_opt, _) in &tool_dirs {
                    if let Some(skills_dir) = skills_dir_opt {
                        let tool_skill_path = skills_dir.join(&dir_name);
                        if tool_skill_path.exists() && fs_utils::is_symlink(&tool_skill_path) {
                            if fs::canonicalize(&tool_skill_path).ok() == skill_canonical {
                                installed_in.push(tool_id.clone());
                            }
                        }
                    }
                }

                skills.push(PluginSkillInfo {
                    dir_name,
                    name,
                    description,
                    skill_path: path.to_string_lossy().to_string(),
                    installed_in,
                });
            }
        }
    }

    // Scan commands
    let commands_dir = plugin_dir.join("commands");
    let mut commands = Vec::new();
    if commands_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&commands_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = entry.file_name().to_string_lossy().to_string();
                if !file_name.ends_with(".md") {
                    continue;
                }

                let command_name = file_name.trim_end_matches(".md").to_string();
                let cmd_canonical = fs::canonicalize(&path).ok();
                let mut installed_in = Vec::new();

                for (tool_id, _, commands_dir_opt) in &tool_dirs {
                    if let Some(cmds_dir) = commands_dir_opt {
                        let tool_cmd_path = cmds_dir.join(&file_name);
                        if tool_cmd_path.exists() && fs_utils::is_symlink(&tool_cmd_path) {
                            if fs::canonicalize(&tool_cmd_path).ok() == cmd_canonical {
                                installed_in.push(tool_id.clone());
                            }
                        }
                    }
                }

                commands.push(PluginCommandInfo {
                    file_name,
                    command_name,
                    file_path: path.to_string_lossy().to_string(),
                    installed_in,
                });
            }
        }
    }

    skills.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    commands.sort_by(|a, b| a.command_name.cmp(&b.command_name));

    Ok(PluginContents { skills, commands })
}

fn resolve_plugin_path(plugin_id: &str) -> Result<PathBuf, String> {
    let reg = storage::load_registry()?;
    let entry = reg
        .plugins
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
    Ok(PathBuf::from(&entry.local_path))
}

fn resolve_tool_commands_dir(tool_id: &str) -> Result<PathBuf, InstallOperationError> {
    let tool_registry = registry::get_tool_registry();
    let entry = tool_registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| InstallOperationError::message(format!("Tool not found: {}", tool_id)))?;

    let config_dir = (entry.dir_resolver)()
        .ok_or_else(|| InstallOperationError::message("Cannot determine home directory"))?;
    let subdir = entry.def.commands_subdir.ok_or_else(|| {
        InstallOperationError::message(format!("Tool {} does not support commands", tool_id))
    })?;

    let commands_dir = config_dir.join(subdir);
    if !commands_dir.exists() {
        fs::create_dir_all(&commands_dir).map_err(|error| {
            InstallOperationError::message(format!(
                "Failed to create command/prompt directory: {}",
                error
            ))
        })?;
    }
    Ok(commands_dir)
}

fn install_plugin_skill_to_skills_dir(
    source: &Path,
    skills_dir: &Path,
    skill_dir_name: &str,
) -> Result<(), InstallOperationError> {
    if !skills_dir.exists() {
        fs::create_dir_all(skills_dir).map_err(|error| {
            InstallOperationError::message(format!("Failed to create skills directory: {}", error))
        })?;
    }

    let target = skills_dir.join(skill_dir_name);
    fs_utils::create_skill_symlink(source, &target).map_err(InstallOperationError::from)
}

fn install_plugin_command_to_commands_dir(
    source: &Path,
    commands_dir: &Path,
    command_file: &str,
) -> Result<(), InstallOperationError> {
    if !commands_dir.exists() {
        fs::create_dir_all(commands_dir).map_err(|error| {
            InstallOperationError::message(format!(
                "Failed to create command/prompt directory: {}",
                error
            ))
        })?;
    }

    let target = commands_dir.join(command_file);
    fs_utils::create_file_symlink(source, &target).map_err(InstallOperationError::from)
}

fn collect_tool_skills_dirs() -> Vec<(String, PathBuf)> {
    registry::get_tool_registry()
        .iter()
        .filter_map(|entry| {
            let subdir = entry.def.skills_subdir?;
            let config_dir = (entry.dir_resolver)()?;
            if !config_dir.exists() {
                return None;
            }

            Some((entry.def.name.to_string(), config_dir.join(subdir)))
        })
        .collect()
}

fn plugin_skill_source(
    plugin_id: &str,
    skill_dir_name: &str,
) -> Result<PathBuf, InstallOperationError> {
    let plugin_dir = resolve_plugin_path(plugin_id).map_err(InstallOperationError::from)?;
    let source = plugin_dir.join("skills").join(skill_dir_name);
    if !source.exists() {
        return Err(InstallOperationError::message(format!(
            "Skill '{}' not found in plugin",
            skill_dir_name
        )));
    }

    Ok(source)
}

fn plugin_command_source(
    plugin_id: &str,
    command_file: &str,
) -> Result<PathBuf, InstallOperationError> {
    let plugin_dir = resolve_plugin_path(plugin_id).map_err(InstallOperationError::from)?;
    let source = plugin_dir.join("commands").join(command_file);
    if !source.exists() {
        return Err(InstallOperationError::message(format!(
            "Command '{}' not found in plugin",
            command_file
        )));
    }

    Ok(source)
}

fn plugin_skill_dir_names(plugin_dir: &Path) -> Result<Vec<String>, InstallOperationError> {
    let skills_dir = plugin_dir.join("skills");
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }
    if !skills_dir.is_dir() {
        return Err(InstallOperationError::message(format!(
            "Plugin skills path is not a directory: {}",
            skills_dir.display()
        )));
    }

    let mut skill_dir_names = Vec::new();
    let entries = fs::read_dir(&skills_dir).map_err(|error| {
        InstallOperationError::message(format!("Failed to read plugin skills directory: {}", error))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            InstallOperationError::message(format!("Failed to read plugin skills entry: {}", error))
        })?;
        let path = entry.path();
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() && !dir_name.starts_with('.') && !dir_name.starts_with("__") {
            skill_dir_names.push(dir_name);
        }
    }

    skill_dir_names.sort();
    Ok(skill_dir_names)
}

fn plugin_command_files(plugin_dir: &Path) -> Result<Vec<String>, InstallOperationError> {
    let commands_dir = plugin_dir.join("commands");
    if !commands_dir.exists() {
        return Ok(Vec::new());
    }
    if !commands_dir.is_dir() {
        return Err(InstallOperationError::message(format!(
            "Plugin commands path is not a directory: {}",
            commands_dir.display()
        )));
    }

    let mut command_files = Vec::new();
    let entries = fs::read_dir(&commands_dir).map_err(|error| {
        InstallOperationError::message(format!(
            "Failed to read plugin commands directory: {}",
            error
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            InstallOperationError::message(format!(
                "Failed to read plugin commands entry: {}",
                error
            ))
        })?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        if path.is_file() && file_name.ends_with(".md") {
            command_files.push(file_name);
        }
    }

    command_files.sort();
    Ok(command_files)
}

fn install_plugin_skill_to_all_targets(
    source: &Path,
    skill_dir_name: &str,
    tool_targets: &[(String, PathBuf)],
) -> Result<(), InstallOperationError> {
    let mut errors = Vec::new();

    for (tool_name, skills_dir) in tool_targets {
        let target = skills_dir.join(skill_dir_name);
        if target.exists() || fs_utils::is_symlink(&target) {
            continue;
        }

        match install_plugin_skill_to_skills_dir(source, skills_dir, skill_dir_name) {
            Ok(()) => {}
            Err(error) if error.requires_elevation() => return Err(error),
            Err(error) => errors.push(format!("{}: {}", tool_name, error)),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InstallOperationError::message(errors.join("; ")))
    }
}

fn install_plugin_command_to_all_targets(
    source: &Path,
    command_file: &str,
    tool_targets: &[(String, PathBuf)],
) -> Result<(), InstallOperationError> {
    let mut errors = Vec::new();

    for (tool_name, commands_dir) in tool_targets {
        let target = commands_dir.join(command_file);
        if target.exists() || fs_utils::is_symlink(&target) {
            continue;
        }

        match install_plugin_command_to_commands_dir(source, commands_dir, command_file) {
            Ok(()) => {}
            Err(error) if error.requires_elevation() => return Err(error),
            Err(error) => errors.push(format!("{}: {}", tool_name, error)),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InstallOperationError::message(errors.join("; ")))
    }
}

fn install_all_plugin_skills_to_tool_dirs(
    plugin_dir: &Path,
    tool_targets: &[(String, PathBuf)],
) -> Result<(), InstallOperationError> {
    let skill_dir_names = plugin_skill_dir_names(plugin_dir)?;
    let mut errors = Vec::new();

    for skill_dir_name in skill_dir_names {
        let source = plugin_dir.join("skills").join(&skill_dir_name);
        match install_plugin_skill_to_all_targets(&source, &skill_dir_name, tool_targets) {
            Ok(()) => {}
            Err(error) if error.requires_elevation() => return Err(error),
            Err(error) => errors.push(format!("{}: {}", skill_dir_name, error)),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InstallOperationError::message(errors.join("; ")))
    }
}

fn install_all_plugin_commands_to_tool_dirs(
    plugin_dir: &Path,
    tool_targets: &[(String, PathBuf)],
) -> Result<(), InstallOperationError> {
    let command_files = plugin_command_files(plugin_dir)?;
    let mut errors = Vec::new();

    for command_file in command_files {
        let source = plugin_dir.join("commands").join(&command_file);
        match install_plugin_command_to_all_targets(&source, &command_file, tool_targets) {
            Ok(()) => {}
            Err(error) if error.requires_elevation() => return Err(error),
            Err(error) => errors.push(format!("{}: {}", command_file, error)),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InstallOperationError::message(errors.join("; ")))
    }
}

pub(crate) fn install_plugin_skill_action(
    plugin_id: &str,
    skill_dir_name: &str,
    tool_id: &str,
) -> Result<(), InstallOperationError> {
    let source = plugin_skill_source(plugin_id, skill_dir_name)?;
    let skills_dir = skill_commands::resolve_tool_skills_dir(tool_id)?;
    install_plugin_skill_to_skills_dir(&source, &skills_dir, skill_dir_name)
}

pub(crate) fn install_plugin_skill_to_all_action(
    plugin_id: &str,
    skill_dir_name: &str,
) -> Result<(), InstallOperationError> {
    let source = plugin_skill_source(plugin_id, skill_dir_name)?;
    let tool_targets = collect_tool_skills_dirs();
    install_plugin_skill_to_all_targets(&source, skill_dir_name, &tool_targets)
}

pub(crate) fn install_plugin_command_action(
    plugin_id: &str,
    command_file: &str,
    tool_id: &str,
) -> Result<(), InstallOperationError> {
    let source = plugin_command_source(plugin_id, command_file)?;
    let commands_dir = resolve_tool_commands_dir(tool_id)?;
    install_plugin_command_to_commands_dir(&source, &commands_dir, command_file)
}

pub(crate) fn install_plugin_command_to_all_action(
    plugin_id: &str,
    command_file: &str,
) -> Result<(), InstallOperationError> {
    let source = plugin_command_source(plugin_id, command_file)?;
    let tool_targets = collect_tool_commands_dirs();
    install_plugin_command_to_all_targets(&source, command_file, &tool_targets)
}

pub(crate) fn install_all_plugin_skills_to_all_tools_action(
    plugin_id: &str,
) -> Result<(), InstallOperationError> {
    let plugin_dir = resolve_plugin_path(plugin_id).map_err(InstallOperationError::from)?;
    let tool_targets = collect_tool_skills_dirs();
    install_all_plugin_skills_to_tool_dirs(&plugin_dir, &tool_targets)
}

pub(crate) fn install_all_plugin_commands_to_all_tools_action(
    plugin_id: &str,
) -> Result<(), InstallOperationError> {
    let plugin_dir = resolve_plugin_path(plugin_id).map_err(InstallOperationError::from)?;
    let tool_targets = collect_tool_commands_dirs();
    install_all_plugin_commands_to_tool_dirs(&plugin_dir, &tool_targets)
}

#[tauri::command]
pub fn install_plugin_skill(
    plugin_id: String,
    skill_dir_name: String,
    tool_id: String,
) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallPluginSkill {
        plugin_id: plugin_id.clone(),
        skill_dir_name: skill_dir_name.clone(),
        tool_id: tool_id.clone(),
    };

    execute_with_optional_elevation(action, || {
        install_plugin_skill_action(&plugin_id, &skill_dir_name, &tool_id)
    })
}

#[tauri::command]
pub fn install_plugin_skill_to_all(
    plugin_id: String,
    skill_dir_name: String,
) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallPluginSkillToAll {
        plugin_id: plugin_id.clone(),
        skill_dir_name: skill_dir_name.clone(),
    };

    execute_with_optional_elevation(action, || {
        install_plugin_skill_to_all_action(&plugin_id, &skill_dir_name)
    })
}

#[tauri::command]
pub fn install_plugin_command(
    plugin_id: String,
    command_file: String,
    tool_id: String,
) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallPluginCommand {
        plugin_id: plugin_id.clone(),
        command_file: command_file.clone(),
        tool_id: tool_id.clone(),
    };

    execute_with_optional_elevation(action, || {
        install_plugin_command_action(&plugin_id, &command_file, &tool_id)
    })
}

#[tauri::command]
pub fn install_plugin_command_to_all(
    plugin_id: String,
    command_file: String,
) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallPluginCommandToAll {
        plugin_id: plugin_id.clone(),
        command_file: command_file.clone(),
    };

    execute_with_optional_elevation(action, || {
        install_plugin_command_to_all_action(&plugin_id, &command_file)
    })
}

#[tauri::command]
pub fn install_all_plugin_skills_to_all_tools(plugin_id: String) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallAllPluginSkillsToAllTools {
        plugin_id: plugin_id.clone(),
    };

    execute_with_optional_elevation(action, || {
        install_all_plugin_skills_to_all_tools_action(&plugin_id)
    })
}

#[tauri::command]
pub fn install_all_plugin_commands_to_all_tools(plugin_id: String) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallAllPluginCommandsToAllTools {
        plugin_id: plugin_id.clone(),
    };

    execute_with_optional_elevation(action, || {
        install_all_plugin_commands_to_all_tools_action(&plugin_id)
    })
}

#[tauri::command]
pub fn remove_plugin_skill(
    plugin_id: String,
    skill_dir_name: String,
    tool_id: String,
) -> Result<(), String> {
    // Validate plugin exists
    let _ = resolve_plugin_path(&plugin_id)?;
    let skills_dir =
        skill_commands::resolve_tool_skills_dir(&tool_id).map_err(|error| error.to_string())?;
    let target = skills_dir.join(&skill_dir_name);

    if !target.exists() && !fs_utils::is_symlink(&target) {
        return Err(format!(
            "Skill '{}' is not installed in {}",
            skill_dir_name, tool_id
        ));
    }

    fs_utils::remove_skill_dir(&target)
}

#[tauri::command]
pub fn remove_plugin_command(
    plugin_id: String,
    command_file: String,
    tool_id: String,
) -> Result<(), String> {
    let _ = resolve_plugin_path(&plugin_id)?;
    let commands_dir = resolve_tool_commands_dir(&tool_id).map_err(|error| error.to_string())?;
    let target = commands_dir.join(&command_file);

    if !target.exists() && !fs_utils::is_symlink(&target) {
        return Err(format!(
            "Command '{}' is not installed in {}",
            command_file, tool_id
        ));
    }

    fs_utils::remove_file_or_symlink(&target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "ai-manager-plugin-commands-{}-{}-{}",
            name,
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&dir).expect("test directory should be created");
        dir
    }

    fn assert_install_result(result: Result<(), InstallOperationError>) {
        match result {
            Ok(()) => {}
            Err(error) if cfg!(windows) && error.requires_elevation() => {}
            Err(error) => panic!("unexpected install failure: {}", error),
        }
    }

    #[test]
    fn install_plugin_skill_to_skills_dir_creates_link_when_possible() {
        let base_dir = test_dir("skill-install");
        let source = base_dir.join("plugin").join("skills").join("alpha");
        let target_dir = base_dir.join("tools").join("codex").join("skills");

        fs::create_dir_all(&source).expect("plugin skill dir should exist");
        fs::write(source.join("SKILL.md"), "# plugin skill").expect("skill file should exist");

        let result = install_plugin_skill_to_skills_dir(&source, &target_dir, "alpha");
        assert_install_result(result);

        let target = target_dir.join("alpha");
        if target.exists() {
            assert!(fs_utils::is_symlink(&target) || target.is_dir());
        }

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn install_plugin_command_to_commands_dir_creates_link_when_possible() {
        let base_dir = test_dir("command-install");
        let source = base_dir.join("plugin").join("commands").join("hello.md");
        let target_dir = base_dir.join("tools").join("claude").join("commands");

        fs::create_dir_all(source.parent().expect("command dir should exist"))
            .expect("command dir should be created");
        fs::write(&source, "# command").expect("command file should exist");

        let result = install_plugin_command_to_commands_dir(&source, &target_dir, "hello.md");
        assert_install_result(result);

        let target = target_dir.join("hello.md");
        if target.exists() {
            assert!(fs_utils::is_symlink(&target) || target.is_file());
        }

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn install_all_plugin_skills_to_tool_dirs_installs_each_skill_once() {
        let base_dir = test_dir("all-skills");
        let plugin_dir = base_dir.join("plugin");
        let skills_root = plugin_dir.join("skills");
        let tool_a_dir = base_dir.join("tools").join("codex").join("skills");
        let tool_b_dir = base_dir.join("tools").join("gemini").join("skills");

        fs::create_dir_all(skills_root.join("alpha")).expect("alpha dir should exist");
        fs::create_dir_all(skills_root.join("beta")).expect("beta dir should exist");
        fs::write(skills_root.join("alpha").join("SKILL.md"), "# alpha")
            .expect("alpha skill should exist");
        fs::write(skills_root.join("beta").join("SKILL.md"), "# beta")
            .expect("beta skill should exist");

        let tool_targets = vec![
            ("Codex".to_string(), tool_a_dir.clone()),
            ("Gemini".to_string(), tool_b_dir.clone()),
        ];

        let result = install_all_plugin_skills_to_tool_dirs(&plugin_dir, &tool_targets);
        assert_install_result(result);

        for skills_dir in [&tool_a_dir, &tool_b_dir] {
            for skill_name in ["alpha", "beta"] {
                let target = skills_dir.join(skill_name);
                if target.exists() {
                    assert!(fs_utils::is_symlink(&target) || target.is_dir());
                }
            }
        }

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn install_all_plugin_commands_to_tool_dirs_installs_each_command_once() {
        let base_dir = test_dir("all-commands");
        let plugin_dir = base_dir.join("plugin");
        let commands_root = plugin_dir.join("commands");
        let tool_dir = base_dir.join("tools").join("claude").join("commands");

        fs::create_dir_all(&commands_root).expect("commands root should exist");
        fs::write(commands_root.join("first.md"), "# first").expect("first command should exist");
        fs::write(commands_root.join("second.md"), "# second")
            .expect("second command should exist");

        let tool_targets = vec![("Claude Code".to_string(), tool_dir.clone())];

        let result = install_all_plugin_commands_to_tool_dirs(&plugin_dir, &tool_targets);
        assert_install_result(result);

        for command_file in ["first.md", "second.md"] {
            let target = tool_dir.join(command_file);
            if target.exists() {
                assert!(fs_utils::is_symlink(&target) || target.is_file());
            }
        }

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn install_plugin_skill_to_all_targets_skips_existing_targets() {
        let base_dir = test_dir("skip-existing");
        let source = base_dir.join("plugin").join("skills").join("alpha");
        let target_dir = base_dir.join("tools").join("codex").join("skills");
        let existing_target = target_dir.join("alpha");

        fs::create_dir_all(&source).expect("plugin skill dir should exist");
        fs::create_dir_all(&existing_target).expect("existing target should exist");

        let tool_targets = vec![("Codex".to_string(), target_dir)];
        let result = install_plugin_skill_to_all_targets(&source, "alpha", &tool_targets);

        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&base_dir);
    }
}
