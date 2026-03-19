use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolInfo {
    pub id: String,
    pub name: String,
    pub config_dir: String,
    pub capability: String,
    pub skills_dir: Option<String>,
    pub config_files: Vec<ConfigFileInfo>,
    pub skill_count: u32,
    pub detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileInfo {
    pub name: String,
    pub path: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub allowed_tools: Option<String>,
    pub dir_name: String,
    pub dir_path: String,
    pub skill_file_path: String,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub has_references: bool,
    pub has_agents: bool,
    pub has_scripts: bool,
    pub installed_in: Vec<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContent {
    pub frontmatter: HashMap<String, String>,
    pub markdown_body: String,
    pub raw_content: String,
    pub references: Vec<ReferenceFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceFile {
    pub name: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EditorInfo {
    pub id: String,
    pub label: String,
}
