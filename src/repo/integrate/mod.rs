use nabu::Object;

use crate::{
    env::{Environment, RepoEnvironment},
    error::{ArgosError, ArgosResult},
    repo::{config::RepoConfig, integrate::build::build_repo},
};

mod build;

/// Integrates a repo
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
///
/// # Returns
/// Returns false if the integration of a repo failed; May also error depending
pub fn integrate_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
    failed_repos: &[String],
) -> ArgosResult<bool> {
    if !check_for_failed_dependencies(&repo_config.dependencies, failed_repos) {
        return Ok(false);
    }

    let mut one_failed = false;
    let mut first_failed_output = None;
    let mut results = Object::new();

    for command in repo_config.commands.iter() {
        match command.as_str() {
            "build" => {
                let (success, output) = build_repo(repo_env, repo_config)?;
                let out = create_result(success, output);
                results.insert(command.to_string(), out);
                if !success && !one_failed {
                    one_failed = true;
                    first_failed_output = Some(output);
                }
            }
            "test" => test_repo(env, repo_env, repo_config)?,
            "doc" => doc_repo(env, repo_env, repo_config)?,
            "doc-test" => doc_test_repo(env, repo_env, repo_config)?,
            "clippy" => clippy_repo(env, repo_env, repo_config)?,
            "format" => format_repo(env, repo_env, repo_config)?,
            "update" => update_repo(env, repo_env, repo_config)?,
            "license" => license_repo(env, repo_env, repo_config)?,
            _ => {
                return Err(ArgosError::IntegrateRepoError(format!(
                    "Unknown command: {}",
                    command
                )));
            }
        }
    }

    results.insert("all_succeeded".to_string(), !one_failed);
    if let Some(output) = first_failed_output {
        results.insert("first_failed_command".to_string(), output);
    }

    // TODO: Save results inside 100 run archive and as latest run

    // TODO: Consider how to deal with `cargo clean` sensibly
    // Size Check: Use a recursive std::fs::read_dir to calculate folder size (or call du -s via Command since it's standard on Linux)
    // Also run on failure to clean up possible stale symbols?
    if one_failed { Ok(false) } else { Ok(true) }
}

fn create_result(success: bool, output: String) -> Object {
    // {
    //  "success": true,
    //  "output": "..."
    // }
    let mut out = Object::new();
    out.insert("success".to_string(), success);
    out.insert("output".to_string(), output);
    out
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
