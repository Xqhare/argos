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

    // Ensure --fix is the first argument
    if let Some(pos) = args.iter().position(|x| x == "--fix") {
        args.remove(pos);
    }
    args.insert(0, "--fix".to_string());

    // Ensure clippy::all and clippy::pedantic are always used
    if !args.contains(&"--".to_string()) {
        args.push("--".to_string());
    }

    // Always append the lints at the end to ensure they are active
    args.push("-W".to_string());
    args.push("clippy::all".to_string());
    args.push("-W".to_string());
    args.push("clippy::pedantic".to_string());

    run_test_and_commit(
        env,
        repo_env,
        repo_config,
        "clippy",
        args,
        "made clippy happy",
    )
}
