use crate::{
    env::RepoEnvironment,
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd, test::test_repo},
    },
    utils::git::git_commit,
};

/// Runs `cargo update` on a repo
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the updateting was successful and a string containing the output
pub fn update_repo(
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let args = get_repo_args(repo_config, "update");
    let (first_success, _) = test_repo(repo_env, repo_config)?;
    if first_success {
        let (success, output) = run_cargo_cmd(repo_env, "update", args)?;
        if success {
            if test_repo(repo_env, repo_config)?.0 {
                // All good
                git_commit(&repo_env.repo_path, "update", "ran cargo update")?;
                return Ok((true, output));
            } else {
                // Not all tests pass after update
                return Ok((false, output));
            }
        } else {
            // Not all good after running update
            return Ok((false, output));
        }
    } else {
        return Ok((false, "First testing pass failed - aborted.".to_string()));
    }
}
