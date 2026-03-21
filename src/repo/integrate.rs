use std::path::Path;

use nabu::{Object, XffValue};

use crate::{
    env::{Environment, RepoEnvironment},
    error::{ArgosError, ArgosResult},
    repo::config::RepoConfig,
};

/// Integrates a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
///
/// # Returns
/// Returns false if the integration of a repo failed; May also error depending
pub fn integrate_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    failed_repos: &[String],
) -> ArgosResult<bool> {
    let repo_config = repo_config(env, repo_env, failed_repos)?;
    Ok(true)
}

fn repo_config(
    env: &Environment,
    repo_env: &RepoEnvironment,
    failed_repos: &[String],
) -> ArgosResult<()> {
    let repo_config = RepoConfig::new(repo_env);
    Ok(())
}
