use crate::skills::fs_utils;
use crate::skills::models::*;
use crate::skills::parser;
use crate::skills::registry;
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

    let config_dir = (entry.dir_resolver)()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

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
pub fn read_skill(skill_path: String) -> Result<SkillContent, String> {
    let skill_dir = PathBuf::from(&skill_path);
    let skill_file = skill_dir.join("SKILL.md");

    let raw_content = fs::read_to_string(&skill_file)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

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
    let hub_dir = registry::get_hub_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

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
    let hub_dir = registry::get_hub_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

    let source = hub_dir.join(&hub_skill_name);
    if !source.exists() {
        return Err(format!("Hub skill not found: {}", hub_skill_name));
    }

    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir = (entry.dir_resolver)()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

    let skills_subdir = entry
        .def
        .skills_subdir
        .ok_or_else(|| format!("Tool {} does not support skills", tool_id))?;

    let skills_dir = config_dir.join(skills_subdir);

    // Create skills directory if it doesn't exist
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir)
            .map_err(|e| format!("Failed to create skills directory: {}", e))?;
    }

    let target = skills_dir.join(&hub_skill_name);
    fs_utils::create_skill_symlink(&source, &target)
}

#[tauri::command]
pub fn remove_skill(tool_id: String, skill_name: String) -> Result<(), String> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir = (entry.dir_resolver)()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

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
pub fn toggle_skill(tool_id: String, skill_name: String, enabled: bool) -> Result<(), String> {
    let registry = registry::get_tool_registry();
    let entry = registry
        .into_iter()
        .find(|e| e.def.id == tool_id)
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let config_dir = (entry.dir_resolver)()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

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
            fs::rename(&from, &to)
                .map_err(|e| format!("Failed to enable skill: {}", e))?;
        }
    } else {
        // Rename xxx to .disabled-xxx
        let disabled_name = format!(".disabled-{}", skill_name);
        let from = skills_dir.join(&skill_name);
        let to = skills_dir.join(&disabled_name);
        if from.exists() || fs_utils::is_symlink(&from) {
            fs::rename(&from, &to)
                .map_err(|e| format!("Failed to disable skill: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn read_config_file(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read config file: {}", e))
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
