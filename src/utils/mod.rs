pub mod git;

use std::time::Duration;

use athena::sorting::kahns_managed;
use nabu::{Array, XffValue};

use crate::{
    env::{Environment, RepoEnvironment},
    error::{ArgosError, ArgosResult},
    utils::git::latest_git_hash,
};

/// Takes in a list of repositories and returns a list sorted by their dependencies. First element
/// is required by later elements
pub fn sort_repo_list(repo_list: &Array, env: &Environment) -> ArgosResult<Array> {
    let mut repo_owned_dependency_list: Vec<(String, Vec<String>)> = Vec::new();
    for repo in repo_list {
        let repo = repo.into_string().ok_or(ArgosError::XffValue(
            "Repo array must contain only strings as children.".to_string(),
        ))?;
        let repo_env = RepoEnvironment::new(&repo, env)?;
        let mut dependencies = Vec::new();
        if repo_env.repo_advanced_config_path.exists() {
            let config_data = nabu::serde::read(&repo_env.repo_advanced_config_path)
                .map_err(|e| ArgosError::Xff(e.to_string()))?
                .into_object()
                .ok_or(ArgosError::XffValue(
                    "Repo config must be an object.".to_string(),
                ))?;
            if let Some(x) = config_data.get("requires") {
                if let Some(x) = x.into_array() {
                    for dependency in x {
                        let tmp = dependency.into_string().ok_or(ArgosError::XffValue(
                            "Dependencies must be strings.".to_string(),
                        ))?;
                        dependencies.push(tmp);
                    }
                } else {
                    return Err(ArgosError::XffValue(
                        "Dependencies must be an array.".to_string(),
                    ));
                }
            }
        }
        repo_owned_dependency_list.push((repo, dependencies));
    }

    match kahns_managed(&repo_owned_dependency_list) {
        Ok(x) => {
            let out = x
                .iter()
                .map(|x: &String| XffValue::from(x.clone()))
                .collect();
            Ok(out)
        }
        Err(e) => Err(ArgosError::Xff(e)),
    }
}

pub fn was_updated(repo_env: &RepoEnvironment) -> ArgosResult<bool> {
    let latest_hash = latest_git_hash(&repo_env.repo_path)?;
    let mut repo_metadata = match nabu::serde::read(&repo_env.repo_tracking_xff) {
        Ok(xff) => {
            if xff.is_object() {
                xff.into_object().unwrap()
            } else {
                return Err(ArgosError::XffValue(
                    "Repo metadata XFF must be an object.".to_string(),
                ));
            }
        }
        Err(e) => return Err(ArgosError::Xff(e.to_string())),
    };
    let previous_hash = match repo_metadata.get("hash") {
        Some(x) => match x.as_string() {
            Some(x) => x,
            None => {
                return Err(ArgosError::XffValue("Hash must be a string.".to_string()));
            }
        },
        None => {
            return Err(ArgosError::XffValue("Hash must be present.".to_string()));
        }
    };
    let mut overwrite = false;
    if let Some(date) = repo_metadata.get("last_run") {
        let last_run = {
            let tmp = match date.into_unix_timestamp() {
                Some(x) => x,
                None => {
                    return Err(ArgosError::XffValue(
                        "Last run must be a unix timestamp.".to_string(),
                    ));
                }
            };
            horae::Utc::from_timestamp(tmp)
        };
        let now = horae::Utc::now();
        // Salt just to spread the repos out over time a bit - also for the fun of it
        let salt: u64 = repo_env.repo.as_bytes().iter().map(|x| *x as u64).sum();
        // Capped to 15k minutes ~ 10.5 days
        let mut spreadder_mins: u64 = (60 * 24 * 7) + salt;
        if spreadder_mins > 15000 {
            spreadder_mins %= 15000;
        }
        if last_run + Duration::from_mins(spreadder_mins) <= now {
            overwrite = true;
        }
    }
    if &latest_hash == previous_hash && !overwrite {
        return Ok(false);
    }
    repo_metadata.insert("hash", latest_hash);
    nabu::serde::write(&repo_env.repo_tracking_xff, XffValue::from(repo_metadata))
        .map_err(|e| ArgosError::Xff(e.to_string()))?;
    Ok(true)
}

/// Recursively calculates the size of a directory in bytes
pub fn get_dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += get_dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}

/// Gets the uid and gid of the current user
///
/// Returns a string of the form `{uid}:{gid}`
#[allow(clippy::similar_names)]
pub fn get_uid_gid() -> ArgosResult<String> {
    let output_uid = std::process::Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| ArgosError::IntegrateRepoTest(e.to_string()))?;
    let output_uid = String::from_utf8_lossy(&output_uid.stdout)
        .trim()
        .to_string();
    let output_gid = std::process::Command::new("id")
        .arg("-g")
        .output()
        .map_err(|e| ArgosError::IntegrateRepoTest(e.to_string()))?;
    let output_gid = String::from_utf8_lossy(&output_gid.stdout)
        .trim()
        .to_string();
    let output = format!("{output_uid}:{output_gid}");
    Ok(output)
}
