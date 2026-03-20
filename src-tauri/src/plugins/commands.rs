use crate::plugins::github;
use crate::plugins::models::*;
use crate::plugins::storage;
use crate::skills::fs_utils;
use crate::skills::parser;
use crate::skills::registry;
use std::fs;
use std::path::{Path, PathBuf};

fn read_plugin_json(plugin_dir: &Path) -> Result<PluginJson, String> {
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

fn generate_plugin_id(source: &PluginSource, metadata: &PluginMetadata) -> String {
    match source {
        PluginSource::GitHub { owner, repo } => format!("{}--{}", owner, repo),
        PluginSource::Local { .. } => {
            metadata
                .name
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect()
        }
    }
}

fn now_iso8601() -> String {
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
    };

    reg.plugins.push(entry.clone());
    storage::save_registry(&reg)?;

    Ok(entry)
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
            Ok(reg
                .plugins
                .into_iter()
                .find(|p| p.id == plugin_id)
                .unwrap())
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
            Ok(reg
                .plugins
                .into_iter()
                .find(|p| p.id == plugin_id)
                .unwrap())
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

fn resolve_tool_skills_dir(tool_id: &str) -> Result<PathBuf, String> {
    let tool_registry = registry::get_tool_registry();
    let entry = tool_registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir = (entry.dir_resolver)().ok_or("Cannot determine home directory")?;
    let subdir = entry
        .def
        .skills_subdir
        .ok_or_else(|| format!("Tool {} does not support skills", tool_id))?;

    let skills_dir = config_dir.join(subdir);
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir)
            .map_err(|e| format!("Failed to create skills directory: {}", e))?;
    }
    Ok(skills_dir)
}

fn resolve_tool_commands_dir(tool_id: &str) -> Result<PathBuf, String> {
    let tool_registry = registry::get_tool_registry();
    let entry = tool_registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir = (entry.dir_resolver)().ok_or("Cannot determine home directory")?;
    let subdir = entry
        .def
        .commands_subdir
        .ok_or_else(|| format!("Tool {} does not support commands", tool_id))?;

    let commands_dir = config_dir.join(subdir);
    if !commands_dir.exists() {
        fs::create_dir_all(&commands_dir)
            .map_err(|e| format!("Failed to create commands directory: {}", e))?;
    }
    Ok(commands_dir)
}

#[tauri::command]
pub fn install_plugin_skill(
    plugin_id: String,
    skill_dir_name: String,
    tool_id: String,
) -> Result<(), String> {
    let plugin_dir = resolve_plugin_path(&plugin_id)?;
    let source = plugin_dir.join("skills").join(&skill_dir_name);
    if !source.exists() {
        return Err(format!("Skill '{}' not found in plugin", skill_dir_name));
    }

    let skills_dir = resolve_tool_skills_dir(&tool_id)?;
    let target = skills_dir.join(&skill_dir_name);

    fs_utils::create_skill_symlink(&source, &target)
}

#[tauri::command]
pub fn install_plugin_skill_to_all(
    plugin_id: String,
    skill_dir_name: String,
) -> Result<(), String> {
    let plugin_dir = resolve_plugin_path(&plugin_id)?;
    let source = plugin_dir.join("skills").join(&skill_dir_name);
    if !source.exists() {
        return Err(format!("Skill '{}' not found in plugin", skill_dir_name));
    }

    let tool_registry = registry::get_tool_registry();
    let mut errors: Vec<String> = Vec::new();

    for entry in &tool_registry {
        let subdir = match entry.def.skills_subdir {
            Some(s) => s,
            None => continue,
        };
        let config_dir = match (entry.dir_resolver)() {
            Some(d) => d,
            None => continue,
        };
        if !config_dir.exists() {
            continue;
        }

        let skills_dir = config_dir.join(subdir);
        if !skills_dir.exists() {
            if let Err(e) = fs::create_dir_all(&skills_dir) {
                errors.push(format!("{}: {}", entry.def.name, e));
                continue;
            }
        }

        let target = skills_dir.join(&skill_dir_name);
        if target.exists() {
            continue; // Already installed
        }

        if let Err(e) = fs_utils::create_skill_symlink(&source, &target) {
            errors.push(format!("{}: {}", entry.def.name, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[tauri::command]
pub fn install_plugin_command(
    plugin_id: String,
    command_file: String,
    tool_id: String,
) -> Result<(), String> {
    let plugin_dir = resolve_plugin_path(&plugin_id)?;
    let source = plugin_dir.join("commands").join(&command_file);
    if !source.exists() {
        return Err(format!("Command '{}' not found in plugin", command_file));
    }

    let commands_dir = resolve_tool_commands_dir(&tool_id)?;
    let target = commands_dir.join(&command_file);

    fs_utils::create_file_symlink(&source, &target)
}

#[tauri::command]
pub fn install_plugin_command_to_all(
    plugin_id: String,
    command_file: String,
) -> Result<(), String> {
    let plugin_dir = resolve_plugin_path(&plugin_id)?;
    let source = plugin_dir.join("commands").join(&command_file);
    if !source.exists() {
        return Err(format!("Command '{}' not found in plugin", command_file));
    }

    let tool_registry = registry::get_tool_registry();
    let mut errors: Vec<String> = Vec::new();

    for entry in &tool_registry {
        let subdir = match entry.def.commands_subdir {
            Some(s) => s,
            None => continue,
        };
        let config_dir = match (entry.dir_resolver)() {
            Some(d) => d,
            None => continue,
        };
        if !config_dir.exists() {
            continue;
        }

        let commands_dir = config_dir.join(subdir);
        if !commands_dir.exists() {
            if let Err(e) = fs::create_dir_all(&commands_dir) {
                errors.push(format!("{}: {}", entry.def.name, e));
                continue;
            }
        }

        let target = commands_dir.join(&command_file);
        if target.exists() {
            continue;
        }

        if let Err(e) = fs_utils::create_file_symlink(&source, &target) {
            errors.push(format!("{}: {}", entry.def.name, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[tauri::command]
pub fn remove_plugin_skill(
    plugin_id: String,
    skill_dir_name: String,
    tool_id: String,
) -> Result<(), String> {
    // Validate plugin exists
    let _ = resolve_plugin_path(&plugin_id)?;
    let skills_dir = resolve_tool_skills_dir(&tool_id)?;
    let target = skills_dir.join(&skill_dir_name);

    if !target.exists() && !fs_utils::is_symlink(&target) {
        return Err(format!("Skill '{}' is not installed in {}", skill_dir_name, tool_id));
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
    let commands_dir = resolve_tool_commands_dir(&tool_id)?;
    let target = commands_dir.join(&command_file);

    if !target.exists() && !fs_utils::is_symlink(&target) {
        return Err(format!(
            "Command '{}' is not installed in {}",
            command_file, tool_id
        ));
    }

    fs_utils::remove_file_or_symlink(&target)
}
