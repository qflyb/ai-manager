use crate::skills::elevation::{
    execute_with_optional_elevation, ElevatedSymlinkAction, InstallOperationError,
};
use crate::skills::fs_utils;
use crate::skills::models::*;
use crate::skills::parser;
use crate::skills::registry;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn read_skill_from_dir(dir: &Path) -> Option<SkillInfo> {
    let raw_dir_name = dir.file_name()?.to_string_lossy().to_string();
    let disabled = raw_dir_name.starts_with(".disabled-");
    let dir_name = if disabled {
        raw_dir_name.strip_prefix(".disabled-").unwrap().to_string()
    } else {
        raw_dir_name.clone()
    };

    let skill_file = dir.join("SKILL.md");

    let (name, description, allowed_tools) = if skill_file.exists() {
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
            parsed.frontmatter.get("allowed-tools").cloned(),
        )
    } else {
        (dir_name.clone(), String::new(), None)
    };

    let is_symlink = fs_utils::is_symlink(dir);
    let symlink_target = fs_utils::resolve_symlink(dir);

    Some(SkillInfo {
        name,
        description,
        allowed_tools,
        dir_name,
        dir_path: dir.to_string_lossy().to_string(),
        skill_file_path: skill_file.to_string_lossy().to_string(),
        is_symlink,
        symlink_target,
        has_references: dir.join("references").is_dir(),
        has_agents: dir.join("agents").is_dir(),
        has_scripts: dir.join("scripts").is_dir(),
        installed_in: Vec::new(),
        disabled,
    })
}

fn list_skill_dirs(skills_dir: &Path) -> Vec<PathBuf> {
    if !skills_dir.is_dir() {
        return Vec::new();
    }

    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if !path.is_dir() || name_str.starts_with("__") {
                continue;
            }

            // Codex uses .system subdirectory
            if name_str == ".system" {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir() {
                            dirs.push(sub_path);
                        }
                    }
                }
                continue;
            }

            // Include .disabled-* dirs, skip other hidden dirs
            if name_str.starts_with('.') && !name_str.starts_with(".disabled-") {
                continue;
            }

            dirs.push(path);
        }
    }

    dirs
}

fn read_command_from_file(path: &Path) -> Option<CommandInfo> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    if !file_name.ends_with(".md") {
        return None;
    }

    Some(CommandInfo {
        file_name: file_name.clone(),
        command_name: file_name.trim_end_matches(".md").to_string(),
        file_path: path.to_string_lossy().to_string(),
        is_symlink: fs_utils::is_symlink(path),
        symlink_target: fs_utils::resolve_symlink(path),
    })
}

fn list_command_files(commands_dir: &Path) -> Vec<PathBuf> {
    if !commands_dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(commands_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if !file_name.ends_with(".md") {
                continue;
            }

            if path.is_file() || fs_utils::is_symlink(&path) {
                files.push(path);
            }
        }
    }

    files
}

fn collect_commands_from_dir(commands_dir: &Path) -> Vec<CommandInfo> {
    let mut commands: Vec<CommandInfo> = list_command_files(commands_dir)
        .iter()
        .filter_map(|path| read_command_from_file(path))
        .collect();

    commands.sort_by(|a, b| a.command_name.cmp(&b.command_name));
    commands
}

fn has_parent_dir_component(path: &Path) -> bool {
    path.components()
        .any(|component| component == std::path::Component::ParentDir)
}

fn read_text_file_with_allowed_dirs(
    path: &Path,
    allowed_dirs: &[PathBuf],
    allowed_dir_label: &str,
    read_label: &str,
) -> Result<String, String> {
    if has_parent_dir_component(path) {
        return Err("Invalid path: path traversal not allowed".to_string());
    }

    if !allowed_dirs.iter().any(|dir| path.starts_with(dir)) {
        return Err(format!(
            "Access denied: path is outside allowed {}",
            allowed_dir_label
        ));
    }

    fs::read_to_string(path).map_err(|error| format!("Failed to read {}: {}", read_label, error))
}

fn validate_command_file_name(command_file: &str) -> Result<(), String> {
    if command_file.is_empty() {
        return Err("Invalid command file name".to_string());
    }

    let path = Path::new(command_file);
    if path.components().count() != 1 || path.file_name() != Some(OsStr::new(command_file)) {
        return Err("Invalid command file name".to_string());
    }

    if !command_file.ends_with(".md") {
        return Err("Command file must end with .md".to_string());
    }

    Ok(())
}

fn resolve_tool_commands_dir(tool_id: &str) -> Result<PathBuf, String> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir =
        (entry.dir_resolver)().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let commands_subdir = entry
        .def
        .commands_subdir
        .ok_or_else(|| format!("Tool {} does not support commands", tool_id))?;

    Ok(config_dir.join(commands_subdir))
}

fn remove_command_from_dir(commands_dir: &Path, command_file: &str) -> Result<(), String> {
    let target = commands_dir.join(command_file);
    if !target.exists() && !fs_utils::is_symlink(&target) {
        return Err(format!("Command not found: {}", command_file));
    }

    fs_utils::remove_file_or_symlink(&target)
}

pub(crate) fn resolve_tool_skills_dir(tool_id: &str) -> Result<PathBuf, InstallOperationError> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| InstallOperationError::message(format!("Tool not found: {}", tool_id)))?;

    let config_dir = (entry.dir_resolver)()
        .ok_or_else(|| InstallOperationError::message("Cannot determine home directory"))?;

    let skills_subdir = entry.def.skills_subdir.ok_or_else(|| {
        InstallOperationError::message(format!("Tool {} does not support skills", tool_id))
    })?;

    let skills_dir = config_dir.join(skills_subdir);
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir).map_err(|error| {
            InstallOperationError::message(format!("Failed to create skills directory: {}", error))
        })?;
    }

    Ok(skills_dir)
}

pub(crate) fn install_hub_skill_to_skills_dir(
    source: &Path,
    skills_dir: &Path,
    hub_skill_name: &str,
) -> Result<(), InstallOperationError> {
    if !skills_dir.exists() {
        fs::create_dir_all(skills_dir).map_err(|error| {
            InstallOperationError::message(format!("Failed to create skills directory: {}", error))
        })?;
    }

    let target = skills_dir.join(hub_skill_name);
    fs_utils::create_skill_symlink(source, &target).map_err(InstallOperationError::from)
}

pub(crate) fn install_skill_action(
    hub_skill_name: &str,
    tool_id: &str,
) -> Result<(), InstallOperationError> {
    let hub_dir = registry::get_hub_dir()
        .ok_or_else(|| InstallOperationError::message("Cannot determine home directory"))?;

    let source = hub_dir.join(hub_skill_name);
    if !source.exists() {
        return Err(InstallOperationError::message(format!(
            "Hub skill not found: {}",
            hub_skill_name
        )));
    }

    let skills_dir = resolve_tool_skills_dir(tool_id)?;
    install_hub_skill_to_skills_dir(&source, &skills_dir, hub_skill_name)
}

#[tauri::command]
pub fn scan_ai_tools() -> Result<Vec<AiToolInfo>, String> {
    let registry = registry::get_tool_registry();
    let mut tools = Vec::new();

    for entry in registry {
        let config_dir = match (entry.dir_resolver)() {
            Some(dir) => dir,
            None => continue,
        };

        let detected = config_dir.exists();
        let mut config_files = Vec::new();
        let mut skills_dir_path: Option<String> = None;
        let commands_dir_path = entry
            .def
            .commands_subdir
            .map(|subdir| config_dir.join(subdir).to_string_lossy().to_string());
        let mut skill_count = 0u32;

        if detected {
            // Check config files
            for (file_name, format) in entry.def.config_files {
                let file_path = config_dir.join(file_name);
                if file_path.exists() {
                    config_files.push(ConfigFileInfo {
                        name: file_name.to_string(),
                        path: file_path.to_string_lossy().to_string(),
                        format: format.to_string(),
                    });
                }
            }

            // Check skills directory
            if let Some(subdir) = entry.def.skills_subdir {
                let skills_dir = config_dir.join(subdir);
                if skills_dir.exists() {
                    skill_count = list_skill_dirs(&skills_dir).len() as u32;
                    skills_dir_path = Some(skills_dir.to_string_lossy().to_string());
                }
            }
        }

        tools.push(AiToolInfo {
            id: entry.def.id.to_string(),
            name: entry.def.name.to_string(),
            config_dir: config_dir.to_string_lossy().to_string(),
            capability: entry.def.capability.to_string(),
            skills_dir: skills_dir_path,
            commands_dir: commands_dir_path,
            config_files,
            skill_count,
            detected,
        });
    }

    Ok(tools)
}

#[tauri::command]
pub fn list_skills(tool_id: String) -> Result<Vec<SkillInfo>, String> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir =
        (entry.dir_resolver)().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let skills_subdir = entry
        .def
        .skills_subdir
        .ok_or_else(|| format!("Tool {} does not have a skills directory", tool_id))?;

    let skills_dir = config_dir.join(skills_subdir);
    let skill_dirs = list_skill_dirs(&skills_dir);

    let skills: Vec<SkillInfo> = skill_dirs
        .iter()
        .filter_map(|dir| read_skill_from_dir(dir))
        .collect();

    Ok(skills)
}

#[tauri::command]
pub fn list_commands(tool_id: String) -> Result<Vec<CommandInfo>, String> {
    let commands_dir = resolve_tool_commands_dir(&tool_id)?;
    Ok(collect_commands_from_dir(&commands_dir))
}

#[tauri::command]
pub fn list_all_skills() -> Result<Vec<SkillGroup>, String> {
    let registry = registry::get_tool_registry();
    let mut groups: HashMap<String, SkillGroup> = HashMap::new();

    for entry in &registry {
        let skills_subdir = match entry.def.skills_subdir {
            Some(s) => s,
            None => continue,
        };
        let config_dir = match (entry.dir_resolver)() {
            Some(d) => d,
            None => continue,
        };
        let skills_dir = config_dir.join(skills_subdir);
        if !skills_dir.is_dir() {
            continue;
        }

        for dir in list_skill_dirs(&skills_dir) {
            if let Some(skill) = read_skill_from_dir(&dir) {
                let tool_entry = SkillToolEntry {
                    tool_id: entry.def.id.to_string(),
                    tool_name: entry.def.name.to_string(),
                    dir_path: skill.dir_path.clone(),
                    skill_file_path: skill.skill_file_path.clone(),
                    is_symlink: skill.is_symlink,
                    symlink_target: skill.symlink_target.clone(),
                    disabled: skill.disabled,
                };

                groups
                    .entry(skill.dir_name.clone())
                    .and_modify(|g| {
                        if !skill.disabled && g.tools.iter().all(|t| t.disabled) {
                            g.name = skill.name.clone();
                            g.description = skill.description.clone();
                            g.has_references = skill.has_references;
                            g.has_agents = skill.has_agents;
                            g.has_scripts = skill.has_scripts;
                        }
                        g.tools.push(tool_entry.clone());
                    })
                    .or_insert_with(|| SkillGroup {
                        dir_name: skill.dir_name,
                        name: skill.name,
                        description: skill.description,
                        has_references: skill.has_references,
                        has_agents: skill.has_agents,
                        has_scripts: skill.has_scripts,
                        tools: vec![tool_entry],
                    });
            }
        }
    }

    let mut result: Vec<SkillGroup> = groups.into_values().collect();
    result.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    Ok(result)
}

#[tauri::command]
pub fn read_skill(skill_path: String) -> Result<SkillContent, String> {
    let skill_dir = PathBuf::from(&skill_path);

    // Reject any path with ".." components
    if skill_dir
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        return Err("Invalid path: path traversal not allowed".to_string());
    }

    // Path must be within hub_dir or a known tool config directory
    let hub_dir = registry::get_hub_dir();
    let tool_dirs: Vec<PathBuf> = registry::get_tool_registry()
        .iter()
        .filter_map(|e| (e.dir_resolver)())
        .collect();
    let is_allowed = hub_dir.as_ref().map_or(false, |d| skill_dir.starts_with(d))
        || tool_dirs.iter().any(|d| skill_dir.starts_with(d));
    if !is_allowed {
        return Err("Access denied: path is outside allowed directories".to_string());
    }
    let skill_file = skill_dir.join("SKILL.md");

    let raw_content =
        fs::read_to_string(&skill_file).map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    let parsed = parser::parse_skill_md(&raw_content);

    // Read reference files
    let references_dir = skill_dir.join("references");
    let mut references = Vec::new();

    if references_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&references_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        references.push(ReferenceFile {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: path.to_string_lossy().to_string(),
                            content,
                        });
                    }
                }
            }
        }
    }

    Ok(SkillContent {
        frontmatter: parsed.frontmatter,
        markdown_body: parsed.body,
        raw_content,
        references,
    })
}

#[tauri::command]
pub fn get_hub_skills() -> Result<Vec<SkillInfo>, String> {
    let hub_dir =
        registry::get_hub_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;

    if !hub_dir.exists() {
        return Ok(Vec::new());
    }

    let registry = registry::get_tool_registry();

    // Build a map of tool_id -> skills_dir for cross-referencing
    let tool_skills_dirs: Vec<(String, PathBuf)> = registry
        .iter()
        .filter_map(|entry| {
            let config_dir = (entry.dir_resolver)()?;
            let subdir = entry.def.skills_subdir?;
            let skills_dir = config_dir.join(subdir);
            if skills_dir.exists() {
                Some((entry.def.id.to_string(), skills_dir))
            } else {
                None
            }
        })
        .collect();

    let mut hub_skills = Vec::new();

    if let Ok(entries) = fs::read_dir(&hub_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name_str = entry.file_name().to_string_lossy().to_string();

            if !path.is_dir() || name_str.starts_with('.') || name_str.starts_with("__") {
                continue;
            }

            if let Some(mut skill) = read_skill_from_dir(&path) {
                // Check which tools have this skill installed (via symlink)
                let hub_skill_canonical = fs::canonicalize(&path).ok();

                for (tool_id, skills_dir) in &tool_skills_dirs {
                    let tool_skill_dir = skills_dir.join(&name_str);
                    if tool_skill_dir.exists() {
                        let is_linked = if fs_utils::is_symlink(&tool_skill_dir) {
                            // Check if symlink points to this hub skill
                            fs::canonicalize(&tool_skill_dir).ok() == hub_skill_canonical
                        } else {
                            false
                        };
                        if is_linked {
                            skill.installed_in.push(tool_id.clone());
                        }
                    }
                }

                hub_skills.push(skill);
            }
        }
    }

    Ok(hub_skills)
}

#[tauri::command]
pub fn install_skill(hub_skill_name: String, tool_id: String) -> Result<(), String> {
    let action = ElevatedSymlinkAction::InstallSkill {
        hub_skill_name: hub_skill_name.clone(),
        tool_id: tool_id.clone(),
    };

    execute_with_optional_elevation(action, || install_skill_action(&hub_skill_name, &tool_id))
}

#[tauri::command]
pub fn remove_skill(tool_id: String, skill_name: String) -> Result<(), String> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir =
        (entry.dir_resolver)().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let skills_subdir = entry
        .def
        .skills_subdir
        .ok_or_else(|| format!("Tool {} does not support skills", tool_id))?;

    let skill_path = config_dir.join(skills_subdir).join(&skill_name);

    if !skill_path.exists() && !fs_utils::is_symlink(&skill_path) {
        return Err(format!("Skill not found: {}", skill_name));
    }

    fs_utils::remove_skill_dir(&skill_path)
}

#[tauri::command]
pub fn remove_skill_from_all(skill_name: String) -> Result<(), String> {
    let registry = registry::get_tool_registry();
    let mut errors: Vec<String> = Vec::new();

    for entry in &registry {
        let skills_subdir = match entry.def.skills_subdir {
            Some(s) => s,
            None => continue,
        };
        let config_dir = match (entry.dir_resolver)() {
            Some(d) => d,
            None => continue,
        };
        let skills_dir = config_dir.join(skills_subdir);
        if !skills_dir.is_dir() {
            continue;
        }

        // Check both enabled and disabled paths
        for candidate in [
            skills_dir.join(&skill_name),
            skills_dir.join(format!(".disabled-{}", skill_name)),
        ] {
            if candidate.exists() || fs_utils::is_symlink(&candidate) {
                if let Err(e) = fs_utils::remove_skill_dir(&candidate) {
                    errors.push(format!(
                        "{} ({}): {}",
                        entry.def.name,
                        candidate.display(),
                        e
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[tauri::command]
pub fn toggle_skill(tool_id: String, skill_name: String, enabled: bool) -> Result<(), String> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir =
        (entry.dir_resolver)().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let skills_subdir = entry
        .def
        .skills_subdir
        .ok_or_else(|| format!("Tool {} does not support skills", tool_id))?;

    let skills_dir = config_dir.join(skills_subdir);

    if enabled {
        // Rename .disabled-xxx to xxx
        let disabled_name = format!(".disabled-{}", skill_name);
        let from = skills_dir.join(&disabled_name);
        let to = skills_dir.join(&skill_name);
        if from.exists() {
            fs::rename(&from, &to).map_err(|e| format!("Failed to enable skill: {}", e))?;
        }
    } else {
        // Rename xxx to .disabled-xxx
        let disabled_name = format!(".disabled-{}", skill_name);
        let from = skills_dir.join(&skill_name);
        let to = skills_dir.join(&disabled_name);
        if from.exists() || fs_utils::is_symlink(&from) {
            fs::rename(&from, &to).map_err(|e| format!("Failed to disable skill: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn read_config_file(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    let allowed_dirs: Vec<PathBuf> = registry::get_tool_registry()
        .iter()
        .filter_map(|e| (e.dir_resolver)())
        .collect();

    read_text_file_with_allowed_dirs(&path, &allowed_dirs, "config directories", "config file")
}

#[tauri::command]
pub fn read_command_file(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    let allowed_dirs: Vec<PathBuf> = registry::get_tool_registry()
        .iter()
        .filter_map(|entry| {
            let config_dir = (entry.dir_resolver)()?;
            let commands_subdir = entry.def.commands_subdir?;
            Some(config_dir.join(commands_subdir))
        })
        .collect();

    read_text_file_with_allowed_dirs(&path, &allowed_dirs, "command directories", "command file")
}

fn is_command_available(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("where")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[tauri::command]
pub fn detect_editors() -> Vec<EditorInfo> {
    let known_editors = vec![
        ("code", "VSCode"),
        ("code-insiders", "VSCode Insiders"),
        ("cursor", "Cursor"),
        ("windsurf", "Windsurf"),
        ("zed", "Zed"),
        ("webstorm", "WebStorm"),
        ("idea", "IntelliJ IDEA"),
        ("fleet", "Fleet"),
        ("subl", "Sublime Text"),
        ("atom", "Atom"),
        ("notepad++", "Notepad++"),
        ("nvim", "Neovim"),
        ("vim", "Vim"),
        ("antigravity", "Antigravity"),
    ];

    known_editors
        .into_iter()
        .filter(|(cmd, _)| is_command_available(cmd))
        .map(|(cmd, label)| EditorInfo {
            id: cmd.to_string(),
            label: label.to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn open_in_editor(file_path: String, editor: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    std::process::Command::new(&editor)
        .arg(&file_path)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("Editor '{}' not found in PATH.", editor)
            } else {
                format!("Failed to open editor: {}", e)
            }
        })
}

#[tauri::command]
pub fn remove_command(tool_id: String, command_file: String) -> Result<(), String> {
    validate_command_file_name(&command_file)?;
    let commands_dir = resolve_tool_commands_dir(&tool_id)?;
    remove_command_from_dir(&commands_dir, &command_file)
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
            "ai-manager-skills-commands-{}-{}-{}",
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
    fn install_hub_skill_to_skills_dir_creates_link_when_possible() {
        let base_dir = test_dir("install");
        let hub_skill_dir = base_dir.join("hub").join("skill-one");
        let skills_dir = base_dir.join("tool").join("skills");

        fs::create_dir_all(&hub_skill_dir).expect("hub skill dir should exist");
        fs::write(hub_skill_dir.join("SKILL.md"), "# skill").expect("skill file should exist");

        let result = install_hub_skill_to_skills_dir(&hub_skill_dir, &skills_dir, "skill-one");
        assert_install_result(result);

        let target = skills_dir.join("skill-one");
        if target.exists() {
            assert!(fs_utils::is_symlink(&target) || target.is_dir());
        }

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn install_hub_skill_to_skills_dir_reports_existing_target() {
        let base_dir = test_dir("existing-target");
        let hub_skill_dir = base_dir.join("hub").join("skill-one");
        let skills_dir = base_dir.join("tool").join("skills");
        let existing_target = skills_dir.join("skill-one");

        fs::create_dir_all(&hub_skill_dir).expect("hub skill dir should exist");
        fs::create_dir_all(&existing_target).expect("existing target should exist");

        let error = install_hub_skill_to_skills_dir(&hub_skill_dir, &skills_dir, "skill-one")
            .expect_err("existing target should fail");

        assert!(!error.requires_elevation());
        assert!(error.to_string().contains("Target already exists"));

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn collect_commands_from_dir_ignores_non_markdown_entries_and_sorts_results() {
        let base_dir = test_dir("command-list");
        let commands_dir = base_dir.join("commands");

        fs::create_dir_all(commands_dir.join("nested")).expect("nested dir should exist");
        fs::write(commands_dir.join("zeta.md"), "# zeta").expect("zeta command should exist");
        fs::write(commands_dir.join("alpha.md"), "# alpha").expect("alpha command should exist");
        fs::write(commands_dir.join("notes.txt"), "ignore").expect("notes file should exist");
        fs::write(commands_dir.join("nested").join("inside.md"), "# inside")
            .expect("nested file should exist");

        let commands = collect_commands_from_dir(&commands_dir);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command_name, "alpha");
        assert_eq!(commands[1].command_name, "zeta");
        assert!(commands
            .iter()
            .all(|command| command.file_name.ends_with(".md")));

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn collect_commands_from_dir_returns_empty_when_directory_is_missing() {
        let base_dir = test_dir("command-list-missing");
        let missing_dir = base_dir.join("missing");

        let commands = collect_commands_from_dir(&missing_dir);

        assert!(commands.is_empty());

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn collect_commands_from_dir_reports_symlink_metadata_when_possible() {
        let base_dir = test_dir("command-list-symlink");
        let source_dir = base_dir.join("source");
        let commands_dir = base_dir.join("commands");
        let source = source_dir.join("linked.md");
        let target = commands_dir.join("linked.md");

        fs::create_dir_all(&source_dir).expect("source dir should exist");
        fs::create_dir_all(&commands_dir).expect("commands dir should exist");
        fs::write(&source, "# linked").expect("source command should exist");

        match fs_utils::create_file_symlink(&source, &target) {
            Ok(()) => {}
            Err(error) if cfg!(windows) && error.requires_elevation() => return,
            Err(error) => panic!("unexpected symlink failure: {}", error),
        }

        let commands = collect_commands_from_dir(&commands_dir);

        assert_eq!(commands.len(), 1);
        assert!(commands[0].is_symlink);
        assert_eq!(
            commands[0].symlink_target.as_deref(),
            Some(source.to_string_lossy().as_ref())
        );

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn scan_ai_tools_reports_command_dirs_for_supported_tools() {
        let tools = scan_ai_tools().expect("scan should succeed");

        for tool_id in ["claude", "codex"] {
            let tool = tools
                .iter()
                .find(|tool| tool.id == tool_id)
                .expect("supported tool should be present");
            if tool.detected {
                assert!(tool.commands_dir.is_some());
            }
        }
    }

    #[test]
    fn remove_command_from_dir_removes_regular_markdown_file() {
        let base_dir = test_dir("remove-command-file");
        let commands_dir = base_dir.join("commands");
        let target = commands_dir.join("hello.md");

        fs::create_dir_all(&commands_dir).expect("commands dir should exist");
        fs::write(&target, "# hello").expect("command file should exist");

        remove_command_from_dir(&commands_dir, "hello.md")
            .expect("regular command file should be removed");

        assert!(!target.exists());

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn remove_command_from_dir_removes_symlink_when_possible() {
        let base_dir = test_dir("remove-command-symlink");
        let source_dir = base_dir.join("source");
        let commands_dir = base_dir.join("commands");
        let source = source_dir.join("hello.md");
        let target = commands_dir.join("hello.md");

        fs::create_dir_all(&source_dir).expect("source dir should exist");
        fs::create_dir_all(&commands_dir).expect("commands dir should exist");
        fs::write(&source, "# hello").expect("source command should exist");

        match fs_utils::create_file_symlink(&source, &target) {
            Ok(()) => {}
            Err(error) if cfg!(windows) && error.requires_elevation() => return,
            Err(error) => panic!("unexpected symlink failure: {}", error),
        }

        remove_command_from_dir(&commands_dir, "hello.md")
            .expect("symlink command should be removed");

        assert!(!target.exists());
        assert!(source.exists());

        let _ = fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn remove_command_rejects_unsupported_tool() {
        let error = remove_command("gemini".to_string(), "hello.md".to_string())
            .expect_err("gemini should not support commands");

        assert!(error.contains("does not support commands"));
    }

    #[test]
    fn remove_command_rejects_invalid_file_name() {
        let error = remove_command("claude".to_string(), "../hello.md".to_string())
            .expect_err("path traversal should be rejected");

        assert_eq!(error, "Invalid command file name");
    }
}
