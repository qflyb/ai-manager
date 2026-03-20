use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginJson {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    pub author: Option<PluginAuthorOrString>,
    pub license: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginAuthorOrString {
    Struct { name: String, email: Option<String> },
    Plain(String),
}

impl PluginAuthorOrString {
    pub fn display_name(&self) -> String {
        match self {
            PluginAuthorOrString::Struct { name, .. } => name.clone(),
            PluginAuthorOrString::Plain(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginSource {
    Local { path: String },
    GitHub { owner: String, repo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub id: String,
    pub source: PluginSource,
    pub local_path: String,
    pub metadata: PluginMetadata,
    pub added_at: String,
}

/// Flattened metadata for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub repository: Option<String>,
}

impl From<PluginJson> for PluginMetadata {
    fn from(pj: PluginJson) -> Self {
        PluginMetadata {
            name: pj.name,
            description: pj.description,
            version: pj.version,
            author: pj.author.map(|a| a.display_name()),
            license: pj.license,
            keywords: pj.keywords,
            repository: pj.repository,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContents {
    pub skills: Vec<PluginSkillInfo>,
    pub commands: Vec<PluginCommandInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSkillInfo {
    pub dir_name: String,
    pub name: String,
    pub description: String,
    pub skill_path: String,
    pub installed_in: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommandInfo {
    pub file_name: String,
    pub command_name: String,
    pub file_path: String,
    pub installed_in: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryFile {
    pub plugins: Vec<PluginEntry>,
}
