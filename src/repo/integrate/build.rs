use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd},
    },
};

/// Builds a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the build was successful and a string containing the output
pub fn build_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let args = get_repo_args(repo_config, "build");
    run_cargo_cmd(env, repo_env, repo_config, "build", args)
}
