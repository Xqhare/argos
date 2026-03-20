use std::path::Path;

use crate::error::{ArgosError, ArgosResult};

/// Runs git clone and returns the output
pub fn git_clone(repo_git_url: &str, repo_path: &Path) -> ArgosResult<String> {
    let command = "clone";
    let output = std::process::Command::new("git")
        .arg(command)
        .arg(repo_git_url)
        .arg(repo_path)
        .output()
        .map_err(|e| ArgosError::GitError(format!("Failed to {} repo: {}", command, e)))?;
    if !output.status.success() {
        return Err(ArgosError::GitError(format!(
            "Failed to {} repo: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn git_pull(repo_path: &Path) -> ArgosResult<String> {
    let command = "pull";
    let output = std::process::Command::new("git")
        .arg(command)
        .arg(repo_path)
        .output()
        .map_err(|e| ArgosError::GitError(format!("Failed to {} repo: {}", command, e)))?;
    if !output.status.success() {
        return Err(ArgosError::GitError(format!(
            "Failed to {} repo: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Runs git fetch and returns the latest commit hash
pub fn latest_git_hash(repo_path: &Path) -> ArgosResult<String> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("fetch")
        .output()
        .map_err(|e| ArgosError::GitError(format!("Failed to fetch repo: {}", e)))?;
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("origin/HEAD")
        .output()
        .map_err(|e| ArgosError::GitError(format!("Failed to get latest hash: {}", e)))?;
    if !output.status.success() {
        return Err(ArgosError::GitError(format!(
            "Failed to get latest hash: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
