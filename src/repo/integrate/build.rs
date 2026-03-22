use crate::{
    env::RepoEnvironment,
    error::{ArgosError, ArgosResult},
    repo::config::RepoConfig,
};

/// Builds a repo
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the build was successful and a string containing the output
pub fn build_repo(
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    run_cargo_build(repo_env, repo_config)
}

/// Runs `cargo build` on a repo
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the build was successful and a string containing the output
fn run_cargo_build(
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let args = {
        if let Some(obj) = &repo_config.cmd_args {
            if let Some(args) = obj.get("build") {
                if let Some(array) = args.as_array() {
                    array.iter().map(|x| x.to_string()).collect::<Vec<String>>()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };
    let output = std::process::Command::new("cargo")
        .arg("build")
        .args(args)
        .current_dir(&repo_env.repo_path)
        .output()
        .map_err(|e| ArgosError::IntegrateRepoBuildError(e.to_string()))?;
    if output.status.success() {
        let output = String::from_utf8_lossy(&output.stdout).to_string();
        Ok((true, output))
    } else {
        let error_reason = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((false, error_reason))
    }
}
