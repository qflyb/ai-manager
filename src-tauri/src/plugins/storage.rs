use crate::plugins::models::PluginRegistryFile;
use std::fs;
use std::path::PathBuf;

pub fn get_plugins_base_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".agents").join("plugins"))
}

pub fn get_repos_dir() -> Option<PathBuf> {
    get_plugins_base_dir().map(|p| p.join("repos"))
}

fn registry_path() -> Option<PathBuf> {
    get_plugins_base_dir().map(|p| p.join("registry.json"))
}

pub fn load_registry() -> Result<PluginRegistryFile, String> {
    let path = registry_path().ok_or("Cannot determine home directory")?;

    if !path.exists() {
        return Ok(PluginRegistryFile {
            plugins: Vec::new(),
        });
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read registry.json: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse registry.json: {}", e))
}

pub fn save_registry(registry: &PluginRegistryFile) -> Result<(), String> {
    let path = registry_path().ok_or("Cannot determine home directory")?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        }
    }

    let content = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write registry.json: {}", e))
}
