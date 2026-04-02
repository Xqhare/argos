use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd},
    },
};

/// Tests examples in a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the test was successful and a string containing the output
pub fn examples_test_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let mut args = vec!["--examples".to_string()];
    args.extend(get_repo_args(repo_config, "examples-test"));
    run_cargo_cmd(env, repo_env, repo_config, "test", args)
}
