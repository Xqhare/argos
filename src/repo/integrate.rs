use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
};

/// Integrates a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
///
/// # Returns
/// Returns false if the integration of a repo failed; May also error depending
pub fn integrate_repo(env: &Environment, repo_env: &RepoEnvironment) -> ArgosResult<bool> {
    Ok(())
}
