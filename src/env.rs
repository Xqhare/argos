use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::error::{ArgosError, ArgosResult};
use areia::BaseDirs;
use athena::process::{set_ionice_value, set_scheduler, IoNiceClass, SchedulerPolicy};

// I assume a correctly set up git config
const GIT_ROOT_URL: &str = "ssh://git@serverle:2222/Xqhare/";

pub struct RepoEnvironment {
    pub repo: String,
    pub repo_git_url: String,
    pub repo_path: PathBuf,
    pub repo_tracking_xff: PathBuf,
    pub repo_tracking_json: PathBuf,
    pub repo_history_dir: PathBuf,
    pub repo_config_dir_path: PathBuf,
    pub repo_basic_config_path: PathBuf,
    pub repo_advanced_config_path: PathBuf,
}

impl RepoEnvironment {
    pub fn new(repo: &str, env: &Environment) -> ArgosResult<RepoEnvironment> {
        let repo_git_url = format!("{}{}.git", env.git_root_url, repo);
        let repo_path = env.argos_root_path.join(repo);
        let repo_tracking_xff = env.argos_repo_tracking_path.join(format!("{}.xff", repo));
        let repo_tracking_json = env.argos_repo_tracking_path.join(format!("{}.json", repo));
        let repo_history_dir = env.argos_repo_tracking_path.join(repo);
        let repo_config_dir_path = repo_path.join("ArgosCI");
        let repo_basic_config_path = repo_config_dir_path.join("argus.json");
        let repo_advanced_config_path = repo_config_dir_path.join("config.json");

        // Ensure history directory exists
        create_dir_all(&repo_history_dir).map_err(|e| {
            ArgosError::EnvironmentError(format!(
                "Failed to create history directory for {}: {}",
                repo, e
            ))
        })?;

        Ok(RepoEnvironment {
            repo: repo.to_string(),
            repo_git_url,
            repo_path,
            repo_tracking_xff,
            repo_tracking_json,
            repo_history_dir,
            repo_config_dir_path,
            repo_basic_config_path,
            repo_advanced_config_path,
        })
    }
}

pub struct Environment {
    pub git_root_url: String,
    pub argos_root_path: PathBuf,
    pub argos_data_path: PathBuf,
    pub repo_list_path: PathBuf,
    pub argos_repo_tracking_path: PathBuf,
}

impl Environment {
    pub fn new() -> ArgosResult<Environment> {
        let base_dirs = BaseDirs::new().map_err(|e| {
            ArgosError::EnvironmentError(format!("Failed to get base directories: {}", e))
        })?;
        let argos_data_path = base_dirs.data_dir().to_path_buf();

        let repo_list_path = {
            let out = argos_data_path.join("repo_list.json");
            if out.exists() {
                out
            } else {
                return Err(ArgosError::EnvironmentError(format!(
                    "No repo list found at {}. Please create it to start the CI pipeline.",
                    out.display()
                )));
            }
        };

        let argos_root_path = argos_data_path.join("argos");
        let argos_repo_tracking_path = argos_root_path.join("repo_tracking");

        // Ensure root and tracking directories exist
        create_dir_all(&argos_root_path).map_err(|e| {
            ArgosError::EnvironmentError(format!("Failed to create argos root directory: {}", e))
        })?;
        create_dir_all(&argos_repo_tracking_path).map_err(|e| {
            ArgosError::EnvironmentError(format!(
                "Failed to create repo tracking directory: {}",
                e
            ))
        })?;

        setup_process()?;
        Ok(Environment {
            git_root_url: GIT_ROOT_URL.to_string(),
            repo_list_path,
            argos_repo_tracking_path,
            argos_root_path,
            argos_data_path,
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
