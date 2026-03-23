use nabu::{Object, XffValue};

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
    utils::{get_dir_size, git::git_push},
};

mod build;
mod clippy;
mod doc;
mod doc_test;
mod format;
mod license;
mod test;
mod update;

const TARGET_DIR_LIMIT: u64 = 2 * 1024 * 1024 * 1024; // 2 GB

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
    let mut first_failed_output: Option<String> = None;
    let mut results = Object::new();

    for command in repo_config.commands.iter() {
        let (success, output) = match command.as_str() {
            "build" => build_repo(env, repo_env, repo_config)?,
            "test" => test_repo(env, repo_env, repo_config)?,
            "doc" => doc_repo(env, repo_env, repo_config)?,
            "doc-test" => doc_test_repo(env, repo_env, repo_config)?,
            "clippy" => clippy_repo(env, repo_env, repo_config)?,
            "format" => format_repo(env, repo_env, repo_config)?,
            "update" => update_repo(env, repo_env, repo_config)?,
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

    save_results(repo_env, &results)?;

    git_push(&repo_env.repo_path)?;

    // Handle cleanup
    let target_path = repo_env.repo_path.join("target");
    let target_size = get_dir_size(&target_path).unwrap_or(0);

    if one_failed || target_size > TARGET_DIR_LIMIT {
        clean_repo(env, repo_env)?;
    }

    if one_failed { Ok(false) } else { Ok(true) }
}

fn save_results(repo_env: &RepoEnvironment, results: &Object) -> ArgosResult<()> {
    let save100 = save_100_run_archive(results, repo_env);
    let savelate = save_latest_run(results, repo_env);
    if save100.is_ok() && savelate.is_ok() {
        Ok(())
    } else {
        Err(ArgosError::IntegrateRepoError(format!(
            "Failed to save results: \n {} \n {}",
            save100.unwrap_err(),
            savelate.unwrap_err()
        )))
    }
}

// repo_env.repo_tracking_xff (repo_tracking/{repo}.xff)
// repo_env.repo_tracking_json (repo_tracking/{repo}.json)
fn save_latest_run(results: &Object, repo_env: &RepoEnvironment) -> ArgosResult<()> {
    let json = mawu::write_pretty(
        &repo_env.repo_tracking_json,
        XffValue::from(results.clone()),
        2,
    );
    let xff = nabu::serde::write(&repo_env.repo_tracking_xff, XffValue::from(results.clone()));

    if json.is_ok() && xff.is_ok() {
        Ok(())
    } else {
        Err(ArgosError::IntegrateRepoError(format!(
            "Failed to save results: \n {} \n {}",
            json.unwrap_err(),
            xff.unwrap_err()
        )))
    }
}

// repo_env.repo_history_dir (repo_tracking/{repo}/)
// Then use datetime as filename , xff only
fn save_100_run_archive(results: &Object, repo_env: &RepoEnvironment) -> ArgosResult<()> {
    let now = horae::Utc::now().to_string();
    let save100 = nabu::serde::write(
        &repo_env.repo_history_dir.join(now),
        XffValue::from(results.clone()),
    );
    if save100.is_ok() {
        Ok(())
    } else {
        Err(ArgosError::IntegrateRepoError(format!(
            "Failed to save results: {}",
            save100.unwrap_err()
        )))
    }
}

/// Runs `cargo clean` on a repo
fn clean_repo(env: &Environment, repo_env: &RepoEnvironment) -> ArgosResult<()> {
    execute_in_docker("clean", Vec::new(), env, repo_env)?;
    Ok(())
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
    env: &Environment,
    repo_env: &RepoEnvironment,
    _repo_config: &RepoConfig,
    command: &str,
    args: Vec<String>,
) -> ArgosResult<(bool, String)> {
    execute_in_docker(command, args, env, repo_env)
}

/// Executes a command in a docker container
pub fn execute_in_docker(
    cargo_command: &str,
    cargo_args: Vec<String>,
    env: &Environment,
    repo_env: &RepoEnvironment,
) -> ArgosResult<(bool, String)> {
    let dockerfile = find_dockerfile(cargo_command, env, repo_env);
    let image_tag = format!("argos-{}-{}", repo_env.repo, cargo_command);

    // 1. Build the image
    let build_status = std::process::Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&image_tag)
        .arg("-f")
        .arg(&dockerfile)
        .arg(&repo_env.repo_path)
        .output()
        .map_err(|e| {
            ArgosError::IntegrateRepoError(format!("Failed to build docker image: {}", e))
        })?;

    if !build_status.status.success() {
        return Ok((
            false,
            format!(
                "Failed to build docker image: {}",
                String::from_utf8_lossy(&build_status.stderr)
            ),
        ));
    }

    // 2. Run the container
    let host_user = crate::utils::get_uid_gid()?;
    let repo_path = repo_env
        .repo_path
        .to_str()
        .ok_or_else(|| ArgosError::EnvironmentError("Invalid repo path".to_string()))?;
    let cache_path = env
        .argos_cargo_cache_path
        .to_str()
        .ok_or_else(|| ArgosError::EnvironmentError("Invalid cache path".to_string()))?;

    let mut args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "--user".to_string(),
        host_user,
        "-v".to_string(),
        format!("{}:/app", repo_path),
        "-v".to_string(),
        format!("{}:/usr/local/cargo", cache_path),
        "-w".to_string(),
        "/app".to_string(),
        "-e".to_string(),
        "HOME=/tmp".to_string(),
        image_tag,
        "cargo".to_string(),
        cargo_command.to_string(),
    ];
    args.extend(cargo_args);

    let output = std::process::Command::new("docker")
        .args(args)
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

fn find_dockerfile(
    command: &str,
    env: &Environment,
    repo_env: &RepoEnvironment,
) -> std::path::PathBuf {
    let specific = repo_env
        .repo_config_dir_path
        .join(command)
        .join("Dockerfile");
    if specific.exists() {
        specific
    } else {
        let general = repo_env.repo_config_dir_path.join("Dockerfile");
        if general.exists() {
            general
        } else {
            env.default_dockerfile_path.clone()
        }
    }
}
