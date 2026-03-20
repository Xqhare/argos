use std::path::Path;

use nabu::XffValue;

use crate::{
    env::Environment,
    error::ArgosResult,
    utils::{
        git::{git_clone, git_pull},
        was_updated,
    },
};

pub fn setup_repo(env: &Environment, repo: &str) -> ArgosResult<()> {
    let repo_path = env.argos_root_path.join(repo);
    let repo_git_url = format!("{}{}.git", env.git_root_url, repo);

    if repo_path.join(".git").exists() {
        if !was_updated(env, repo, &repo_path)? {
            return Ok(());
        }
        git_pull(&repo_path)?;
        // TODO: update repo metadata
    } else {
        setup_new_repo(env, repo, &repo_path, &repo_git_url)?;
    }
    // TODO: Setup done, run CI stuff and things
    Ok(())
}

fn setup_new_repo(
    env: &Environment,
    repo: &str,
    repo_path: &Path,
    repo_git_url: &str,
) -> ArgosResult<()> {
    let _ = git_clone(&repo_git_url, &repo_path);
    Ok(())
}
