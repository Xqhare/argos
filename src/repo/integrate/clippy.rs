use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd, test::test_repo},
    },
    utils::git::git_commit,
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
    let (first_success, _) = test_repo(env, repo_env, repo_config)?;
    if first_success {
        let (success, output) = run_cargo_cmd(env, repo_env, repo_config, "clippy", args)?;
        if success {
            if test_repo(env, repo_env, repo_config)?.0 {
                // All good
                git_commit(&repo_env.repo_path, "clippy --fix", "made clippy happy")?;
                return Ok((true, output));
            } else {
                // Not all tests pass after clippy fix
                return Ok((false, output));
            }
        } else {
            // Not all good after running clippy fix
            return Ok((false, output));
        }
    } else {
        return Ok((false, "First testing pass failed - aborted.".to_string()));
    }
}
