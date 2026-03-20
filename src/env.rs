use std::path::PathBuf;

use crate::error::{ArgosError, ArgosResult};
use areia::BaseDirs;
use athena::process::{IoNiceClass, SchedulerPolicy, set_ionice_value, set_scheduler};

// I assume a correctly set up git config
const GIT_ROOT_URL: &str = "ssh://git@serverle:2222/Xqhare/";

pub struct RepoEnvironment {
    pub repo: String,
    pub repo_git_url: String,
    pub repo_path: PathBuf,
    pub repo_tracking: PathBuf,
}

impl RepoEnvironment {
    pub fn new(repo: &str, env: &Environment) -> RepoEnvironment {
        let repo_git_url = format!("{}{}.git", env.git_root_url, repo);
        let repo_path = env.argos_root_path.join(repo);
        let repo_tracking = env.argos_repo_tracking_path.join(format!("{}.xff", repo));
        RepoEnvironment {
            repo: repo.to_string(),
            repo_git_url,
            repo_path,
            repo_tracking,
        }
    }
}

pub struct Environment {
    pub git_root_url: String,
    pub argos_root_path: PathBuf,
    pub argos_runtime_path: PathBuf,
    pub repo_list_path: PathBuf,
    pub argos_repo_tracking_path: PathBuf,
}

impl Environment {
    pub fn new() -> ArgosResult<Environment> {
        let base_dirs = BaseDirs::new().map_err(|e| {
            ArgosError::EnvironmentError(format!("Failed to get base directories: {}", e))
        })?;
        let argos_runtime_path = if let Some(dir) = base_dirs.runtime_dir() {
            dir.to_path_buf()
        } else {
            return Err(ArgosError::EnvironmentError(
                "No runtime directory found".to_string(),
            ));
        };
        let repo_list_path = {
            let out = argos_runtime_path.join("repo_list.json");
            if out.exists() {
                out
            } else {
                return Err(ArgosError::EnvironmentError(
                    "No repo list found".to_string(),
                ));
            }
        };
        let argos_root_path = argos_runtime_path.join("argos");
        let argos_repo_tracking_path = argos_root_path.join("repo_tracking");
        setup_process()?;
        Ok(Environment {
            git_root_url: GIT_ROOT_URL.to_string(),
            repo_list_path,
            argos_repo_tracking_path,
            argos_root_path,
            argos_runtime_path,
        })
    }
}

fn setup_process() -> ArgosResult<()> {
    if let Err(e) = set_scheduler(SchedulerPolicy::Idle, 19) {
        return Err(ArgosError::SetupProcessError(format!(
            "Failed to set scheduler: {}",
            e
        )));
    };
    if let Err(e) = set_ionice_value(IoNiceClass::Idle, 0) {
        return Err(ArgosError::SetupProcessError(format!(
            "Failed to set ionice value: {}",
            e
        )));
    };
    Ok(())
}
