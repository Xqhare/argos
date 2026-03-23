use crate::{
    env::RepoEnvironment,
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd},
    },
};

/// Runs `cargo doc --no-deps` on a repo
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the doc was successful and a string containing the output
pub fn doc_repo(
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let mut args = get_repo_args(repo_config, "doc");

    if !args.contains(&"--no-deps".to_string()) {
        args.push("--no-deps".to_string());
    }

    run_cargo_cmd(repo_env, "doc", args)
}
