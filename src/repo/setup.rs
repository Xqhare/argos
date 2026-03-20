use std::path::Path;

use nabu::{Object, XffValue};

use crate::{
    env::Environment,
    error::{ArgosError, ArgosResult},
    utils::{
        git::{git_clone, git_pull, latest_git_hash},
        was_updated,
    },
};

pub fn setup_repo(env: &Environment, repo: &str) -> ArgosResult<()> {
    let repo_path = env.argos_root_path.join(repo);
    let repo_git_url = format!("{}{}.git", env.git_root_url, repo);
    let repo_tracking = env.argos_repo_tracking_path.join(format!("{}.xff", repo));

    if repo_path.join(".git").exists() {
        if !was_updated(env, repo, &repo_path, &repo_tracking)? {
            return Ok(());
        }
        git_pull(&repo_path)?;
    } else {
        setup_new_repo(&repo_path, &repo_git_url, &repo_tracking)?;
    }
    Ok(())
}

fn setup_new_repo(repo_path: &Path, repo_git_url: &str, repo_tracking: &Path) -> ArgosResult<()> {
    let _ = git_clone(&repo_git_url, &repo_path);
    let latest_hash = latest_git_hash(&repo_path)?;
    let metadata = Object::from(vec![("hash".to_string(), XffValue::String(latest_hash))]);
    nabu::serde::write(repo_tracking, XffValue::from(metadata))
        .map_err(|e| ArgosError::XffError(e.to_string()))?;
    Ok(())
}
