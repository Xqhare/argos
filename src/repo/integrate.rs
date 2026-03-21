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
    _env: &Environment,
    _repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
    failed_repos: &[String],
) -> ArgosResult<bool> {
    if !check_for_failed_dependencies(&repo_config.dependencies, failed_repos) {
        return Ok(false);
    }

    // TODO: Consider how to deal with `cargo clean` sensibly
    // Size Check: Use a recursive std::fs::read_dir to calculate folder size (or call du -s via Command since it's standard on Linux)
    // Also run on failure to clean up possible stale symbols
    Ok(true)
}

/// Checks for failed dependencies
///
/// Returns false if a dependency of this repo has failed
fn check_for_failed_dependencies(
    dependencies: &Option<Vec<String>>,
    failed_repos: &[String],
) -> bool {
    if let Some(deps) = dependencies {
        for dependency in deps {
            if failed_repos.contains(dependency) {
                return false;
            }
        }
    }
    true
}
