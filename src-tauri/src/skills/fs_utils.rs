use std::fs;
use std::path::Path;

pub fn is_symlink(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(meta) => meta.file_type().is_symlink(),
        Err(_) => false,
    }
}

pub fn resolve_symlink(path: &Path) -> Option<String> {
    if is_symlink(path) {
        fs::read_link(path)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    }
}

pub fn create_skill_symlink(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        return Err(format!(
            "Target already exists: {}",
            target.display()
        ));
    }

    if !source.exists() {
        return Err(format!(
            "Source does not exist: {}",
            source.display()
        ));
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
            .map_err(|e| format!("Failed to create symlink: {}", e))
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target).map_err(|e| {
            if e.raw_os_error() == Some(1314) {
                "Requires Developer Mode or administrator privileges to create symlinks. \
                 Please enable Developer Mode in Windows Settings > Developer options."
                    .to_string()
            } else {
                format!("Failed to create symlink: {}", e)
            }
        })
    }
}

pub fn create_file_symlink(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        return Err(format!("Target already exists: {}", target.display()));
    }

    if !source.exists() {
        return Err(format!("Source does not exist: {}", source.display()));
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
            .map_err(|e| format!("Failed to create symlink: {}", e))
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(source, target).map_err(|e| {
            if e.raw_os_error() == Some(1314) {
                "Requires Developer Mode or administrator privileges to create symlinks. \
                 Please enable Developer Mode in Windows Settings > Developer options."
                    .to_string()
            } else {
                format!("Failed to create symlink: {}", e)
            }
        })
    }
}

pub fn remove_file_or_symlink(path: &Path) -> Result<(), String> {
    if is_symlink(path) || path.is_file() {
        fs::remove_file(path).map_err(|e| format!("Failed to remove file: {}", e))
    } else {
        Err(format!("Path is not a file or symlink: {}", path.display()))
    }
}

pub fn remove_skill_dir(path: &Path) -> Result<(), String> {
    if is_symlink(path) {
        // Remove symlink itself, not its target
        #[cfg(windows)]
        {
            fs::remove_dir(path)
                .map_err(|e| format!("Failed to remove symlink: {}", e))
        }
        #[cfg(unix)]
        {
            fs::remove_file(path)
                .map_err(|e| format!("Failed to remove symlink: {}", e))
        }
    } else if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove directory: {}", e))
    } else {
        Err(format!("Path is not a directory: {}", path.display()))
    }
}
