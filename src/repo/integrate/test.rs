use crate::{
    env::RepoEnvironment,
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd},
    },
};

/// Tests a repo
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the test was successful and a string containing the output
pub fn test_repo(
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let args = get_repo_args(repo_config, "test");
    run_cargo_cmd(repo_env, "test", args)
}
