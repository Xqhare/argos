use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_test_and_commit},
    },
};

/// Runs `cargo clippy` on a repo
///
/// If possible, also runs `cargo clippy --fix`
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the clippy was successful and a string containing the output
pub fn clippy_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let mut args = get_repo_args(repo_config, "clippy");
    if !args.contains(&"--fix".to_string()) {
        args.insert(0, "--fix".to_string());
    } else {
        if args[0] != "--fix" {
            let fix_pos = args.iter().position(|x| x == "--fix");
            if let Some(index) = fix_pos {
                args.remove(index);
            }
            args.insert(0, "--fix".to_string());
        }
    }
    run_test_and_commit(env, repo_env, repo_config, "clippy", args, "made clippy happy")
}
