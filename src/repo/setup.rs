use crate::{
    env::RepoEnvironment,
    error::ArgosResult,
    utils::{
        git::{git_clone, git_pull},
        was_updated,
    },
};

/// Sets up a new or existing repo
///
/// # Arguments
/// * `repo` - Repo name
/// * `env` - Environment
///
/// # Returns
/// Returns a boolean indicating if the repo was updated
pub fn setup_repo(repo_env: &RepoEnvironment) -> ArgosResult<bool> {
    if repo_env.repo_path.join(".git").exists() {
        if !was_updated(repo_env)? {
            return Ok(false);
        }
        git_pull(&repo_env.repo_path)?;
    } else {
        setup_new_repo(repo_env)?;
    };
    Ok(true)
}

fn setup_new_repo(repo_env: &RepoEnvironment) -> ArgosResult<()> {
    let _ = git_clone(&repo_env.repo_git_url, &repo_env.repo_path)?;
    Ok(())
}
