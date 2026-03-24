use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_test_and_commit},
    },
};

/// Runs `cargo update` on a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the updating was successful and a string containing the output
pub fn update_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let args = get_repo_args(repo_config, "update");
    run_test_and_commit(
        env,
        repo_env,
        repo_config,
        "update",
        args,
        "ran cargo update",
    )
}
