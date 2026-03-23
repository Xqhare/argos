use nabu::Object;

use crate::{
    env::{Environment, RepoEnvironment},
    error::{ArgosError, ArgosResult},
    repo::{
        config::RepoConfig,
        integrate::{
            build::build_repo, clippy::clippy_repo, doc::doc_repo, doc_test::doc_test_repo,
            format::format_repo, license::license_repo, test::test_repo, update::update_repo,
        },
    },
    utils::git::git_push,
};

mod build;
mod clippy;
mod doc;
mod doc_test;
mod format;
mod license;
mod test;
mod update;

/// Integrates a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
///
/// # Returns
/// Returns false if the integration of a repo failed; May also error depending
pub fn integrate_repo(
    _env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
    failed_repos: &[String],
) -> ArgosResult<bool> {
    if !check_for_failed_dependencies(&repo_config.dependencies, failed_repos) {
        return Ok(false);
    }

    let mut one_failed = false;
    let mut first_failed_output: Option<String> = None;
    let mut results = Object::new();

    for command in repo_config.commands.iter() {
        let (success, output) = match command.as_str() {
            "build" => build_repo(repo_env, repo_config)?,
            "test" => test_repo(repo_env, repo_config)?,
            "doc" => doc_repo(repo_env, repo_config)?,
            "doc-test" => doc_test_repo(repo_env, repo_config)?,
            "clippy" => clippy_repo(repo_env, repo_config)?,
            "format" => format_repo(repo_env, repo_config)?,
            "update" => update_repo(repo_env, repo_config)?,
            "license" => license_repo(repo_env, repo_config)?,
            _ => {
                return Err(ArgosError::IntegrateRepoError(format!(
                    "Unknown command: {}",
                    command
                )));
            }
        };

        let out = create_result(success, &output);
        results.insert(command.to_string(), out);
        if !success && !one_failed {
            one_failed = true;
            first_failed_output = Some(output.to_string());
        }
    }

    results.insert("all_succeeded".to_string(), !one_failed);
    if let Some(output) = first_failed_output {
        results.insert("first_failed_command".to_string(), output);
    }

    // TODO: Save results inside 100 run archive and as latest run

    git_push(&repo_env.repo_path)?;

    // TODO: Consider how to deal with `cargo clean` sensibly
    // Size Check: Use a recursive std::fs::read_dir to calculate folder size (or call du -s via Command since it's standard on Linux)
    // Also run on failure to clean up possible stale symbols?
    if one_failed { Ok(false) } else { Ok(true) }
}

fn create_result(success: bool, output: &str) -> Object {
    // {
    //  "success": true,
    //  "output": "..."
    // }
    let mut out = Object::new();
    out.insert("success".to_string(), success);
    out.insert("output".to_string(), output.to_string());
    out
}

/// Helper function to extract args for a command from RepoConfig
fn get_repo_args(repo_config: &RepoConfig, command: &str) -> Vec<String> {
    if let Some(obj) = &repo_config.cmd_args {
        if let Some(args) = obj.get(command) {
            if let Some(array) = args.as_array() {
                return array.iter().map(|x| x.to_string()).collect();
            }
        }
    }
    Vec::new()
}

/// Checks for failed dependencies
///
/// Returns false if a dependency of this repo has failed
fn check_for_failed_dependencies(
    dependencies: &Option<Vec<String>>,
    failed_repos: &[String],
) -> bool {
    if let Some(deps) = dependencies {
        for dependency in deps {
            if failed_repos.contains(dependency) {
                return false;
            }
        }
    }
    true
}

/// Runs `cargo {command}` on a repo
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `command` - Cargo command
/// * `args` - Command arguments
///
/// # Returns
/// Returns a boolean indicating if the test was successful and a string containing the output
pub fn run_cargo_cmd(
    repo_env: &RepoEnvironment,
    command: &str,
    args: Vec<String>,
) -> ArgosResult<(bool, String)> {
    let output = std::process::Command::new("cargo")
        .arg(command)
        .args(args)
        .current_dir(&repo_env.repo_path)
        .output()
        .map_err(|e| ArgosError::IntegrateRepoTestError(e.to_string()))?;
    if output.status.success() {
        let output = String::from_utf8_lossy(&output.stdout).to_string();
        Ok((true, output))
    } else {
        let error_reason = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((false, error_reason))
    }
}
