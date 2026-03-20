use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{integrate::integrate_repo, setup::setup_repo},
};

mod config;
mod integrate;
mod setup;

/// Continously integrates a repo
///
/// Returns false if the integration of a repo failed; This may also error depending
pub fn continously_integrate_repo(
    env: &Environment,
    repo: &str,
    failed_repos: &[String],
) -> ArgosResult<bool> {
    let repo_env = RepoEnvironment::new(repo, env);
    if !setup_repo(&repo_env)? {
        // No new changes since last run
        return Ok(true);
    };
    // TODO: read config, run CI stuff and things - Also remember the failed repo check for dependencies
    if !integrate_repo(env, &repo_env, failed_repos)? {
        return Ok(false);
    };

    // Sleep for 1 minute - overkill, but if any other process wants I/O I yield to
    // them here
    std::thread::sleep(std::time::Duration::from_secs(60));
    Ok(true)
}
