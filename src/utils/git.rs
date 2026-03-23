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
        .map_err(|e| ArgosError::Git(format!("Failed to {command} repo: {e}")))?;
    if !output.status.success() {
        return Err(ArgosError::Git(format!(
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
        .arg("-C")
        .arg(repo_path)
        .arg(command)
        .output()
        .map_err(|e| ArgosError::Git(format!("Failed to {command} repo: {e}")))?;
    if !output.status.success() {
        return Err(ArgosError::Git(format!(
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
        .map_err(|e| ArgosError::Git(format!("Failed to fetch repo: {e}")))?;
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("origin/HEAD")
        .output()
        .map_err(|e| ArgosError::Git(format!("Failed to get latest hash: {e}")))?;
    if !output.status.success() {
        return Err(ArgosError::Git(format!(
            "Failed to get latest hash: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn latest_git_commit_year(repo_path: &Path) -> ArgosResult<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("-1")
        .arg("--format=%ci")
        .output()
        .map_err(|e| ArgosError::Git(format!("Failed to get latest commit year: {e}")))?;
    if !output.status.success() {
        return Err(ArgosError::Git(format!(
            "Failed to get latest commit year: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let tmp = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // > git log -1 --format=%ci
    // 2026-03-19 21:34:09 +0100

    Ok(tmp[0..4].to_string())
}

/// Runs git commit for all files
///
/// # Arguments
/// * `repo_path` - Path to repo
/// * `command` - Command
/// * `message` - Message
///
/// These are used to generate the commit message:
/// `ArgosCI: <command>: <message>`
///
/// # Returns
/// Returns `Ok` if successful
pub fn git_commit(repo_path: &Path, command: &str, message: &str) -> ArgosResult<()> {
    let message = format!("ArgosCI: {command}: {message}");
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("commit")
        .arg("-a")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(|e| ArgosError::Git(format!("Failed to commit: {e}")))?;
    if !output.status.success() {
        return Err(ArgosError::Git(format!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

/// Runs git push
///
/// # Arguments
/// * `repo_path` - Path to repo
///
/// # Returns
/// Returns `Ok` if successful
pub fn git_push(repo_path: &Path) -> ArgosResult<()> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("push")
        .output()
        .map_err(|e| ArgosError::Git(format!("Failed to push: {e}")))?;
    if !output.status.success() {
        return Err(ArgosError::Git(format!(
            "Failed to push: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}
