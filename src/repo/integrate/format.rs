use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd, test::test_repo},
    },
    utils::git::git_commit,
};

/// Runs `cargo format` on a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the formatting was successful and a string containing the output
pub fn format_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let args = get_repo_args(repo_config, "format");
    let (first_success, _) = test_repo(env, repo_env, repo_config)?;
    if first_success {
        let (success, output) = run_cargo_cmd(env, repo_env, repo_config, "format", args)?;
        if success {
            if test_repo(env, repo_env, repo_config)?.0 {
                // All good
                git_commit(&repo_env.repo_path, "format", "ran cargo format")?;
                return Ok((true, output));
            } else {
                // Not all tests pass after format
                return Ok((false, output));
            }
        } else {
            // Not all good after running format
            return Ok((false, output));
        }
    } else {
        return Ok((false, "First testing pass failed - aborted.".to_string()));
    }
}
