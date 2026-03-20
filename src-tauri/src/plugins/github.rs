use std::path::Path;
use std::process::Command;

pub fn is_git_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg("git")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg("git")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub fn clone_repo(owner: &str, repo: &str, target_dir: &Path) -> Result<(), String> {
    if !is_git_available() {
        return Err(
            "Git is not installed or not found in PATH. Please install Git to add GitHub plugins."
                .to_string(),
        );
    }

    let url = format!("https://github.com/{}/{}.git", owner, repo);

    let output = Command::new("git")
        .args(["clone", "--depth", "1", &url])
        .arg(target_dir)
        .output()
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {}", stderr.trim()));
    }

    Ok(())
}

pub fn pull_repo(repo_dir: &Path) -> Result<(), String> {
    if !is_git_available() {
        return Err("Git is not installed or not found in PATH.".to_string());
    }

    let output = Command::new("git")
        .args(["-C", &repo_dir.to_string_lossy(), "pull", "--ff-only"])
        .output()
        .map_err(|e| format!("Failed to run git pull: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git pull failed: {}", stderr.trim()));
    }

    Ok(())
}
