use crate::{
    env::{Environment, RepoEnvironment},
    error::ArgosResult,
    repo::{
        config::RepoConfig,
        integrate::{get_repo_args, run_cargo_cmd},
    },
};

/// Runs `cargo test --doc` on a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the doc_test was successful and a string containing the output
pub fn doc_test_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let mut args = get_repo_args(repo_config, "doc_test");

    if !args.contains(&"--doc".to_string()) {
        args.insert(0, "--doc".to_string());
    } else {
        if args[0] != "--doc" {
            let doc_pos = args.iter().position(|x| x == "--doc");
            if let Some(index) = doc_pos {
                args.remove(index);
            }
            args.insert(0, "--doc".to_string());
        }
    }

    run_cargo_cmd(env, repo_env, repo_config, "test", args)
}
