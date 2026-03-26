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
    utils::{
        get_dir_size,
        git::{git_commit, git_push},
    },
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
    if !check_for_failed_dependencies(repo_config.dependencies.as_ref(), failed_repos) {
        return Ok(false);
    }

    let mut one_failed = false;
    let mut first_failed_output: Option<String> = None;
    let mut results = Object::new();

    for command in &repo_config.commands {
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
                return Err(ArgosError::IntegrateRepo(format!(
                    "Unknown command: {command}"
                )));
            }
        };

        let out = create_result(success, &output);
        results.insert(command.clone(), out);
        if !success && !one_failed {
            one_failed = true;
            first_failed_output = Some(output.clone());
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
        clean_repo(env, repo_env, repo_config)?;
    }

    if one_failed { Ok(false) } else { Ok(true) }
}

/// Saves a failed integration that occurred before the command pipeline started
pub fn save_failed_integration(repo_env: &RepoEnvironment, error: &str) -> ArgosResult<()> {
    let mut results = Object::new();
    results.insert("all_succeeded".to_string(), false);
    results.insert("setup_error".to_string(), error.to_string());
    save_results(repo_env, &results)
}

fn save_results(repo_env: &RepoEnvironment, results: &Object) -> ArgosResult<()> {
    let mut results_with_metadata = results.clone();

    // Read existing metadata to preserve hash
    if let Ok(xff) = nabu::serde::read(&repo_env.repo_tracking_xff) {
        if let Some(obj) = xff.into_object() {
            if let Some(hash) = obj.get("hash") {
                results_with_metadata.insert("hash".to_string(), hash.clone());
            }
        }
    }
    results_with_metadata.insert(
        "last_run",
        XffValue::from_unix_timestamp(horae::Utc::now().unix_timestamp()),
    );

    let save100 = save_100_run_archive(&results_with_metadata, repo_env);
    let savelate = save_latest_run(&results_with_metadata, repo_env);
    match (save100, savelate) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(e1), Ok(_)) => Err(e1),
        (Ok(_), Err(e2)) => Err(e2),
        (Err(e1), Err(e2)) => Err(ArgosError::IntegrateRepo(format!(
            "Failed to save results: \n {e1} \n {e2}"
        ))),
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

    match (json, xff) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(e1), Ok(_)) => Err(ArgosError::IntegrateRepo(format!("{e1}"))),
        (Ok(_), Err(e2)) => Err(ArgosError::IntegrateRepo(format!("{e2}"))),
        (Err(e1), Err(e2)) => Err(ArgosError::IntegrateRepo(format!("{e1} \n {e2}"))),
    }
}

// repo_env.repo_history_dir (repo_tracking/{repo}/)
// Then use datetime as filename , xff only
fn save_100_run_archive(results: &Object, repo_env: &RepoEnvironment) -> ArgosResult<()> {
    let now = horae::Utc::now().to_string();
    let save100 = nabu::serde::write(
        repo_env.repo_history_dir.join(now),
        XffValue::from(results.clone()),
    );
    if let Err(e) = save100 {
        return Err(ArgosError::IntegrateRepo(format!(
            "Failed to save results: {e}",
        )));
    }

    // Prune to 100 files
    let mut files = std::fs::read_dir(&repo_env.repo_history_dir)
        .map_err(|e| ArgosError::IntegrateRepo(format!("Failed to read history dir: {e}")))?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .collect::<Vec<_>>();

    if files.len() > 100 {
        files.sort();
        for file in files.iter().take(files.len() - 100) {
            let _ = std::fs::remove_file(file);
        }
    }

    Ok(())
}

/// Runs `cargo clean` on a repo
fn clean_repo(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<()> {
    execute_in_docker("clean", Vec::new(), env, repo_env, repo_config)?;
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

/// Helper function to extract args for a command from `RepoConfig`
fn get_repo_args(repo_config: &RepoConfig, command: &str) -> Vec<String> {
    if let Some(obj) = &repo_config.cmd_args
        && let Some(args) = obj.get(command)
        && let Some(array) = args.as_array()
    {
        return array.iter().map(std::string::ToString::to_string).collect();
    }
    Vec::new()
}

/// Checks for failed dependencies
///
/// Returns false if a dependency of this repo has failed
fn check_for_failed_dependencies(
    dependencies: Option<&Vec<String>>,
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

/// Runs a cargo command and commits on success after ensuring all tests pass.
///
/// # Arguments
/// * `env` - Environment
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
/// * `cargo_command` - Cargo command
/// * `args` - Command arguments
/// * `commit_msg` - Git commit message
///
/// # Returns
/// Returns a boolean indicating if the command was successful and a string containing the output
pub fn run_test_and_commit(
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
    cargo_command: &str,
    args: Vec<String>,
    commit_msg: &str,
) -> ArgosResult<(bool, String)> {
    let (first_success, _) = test_repo(env, repo_env, repo_config)?;
    if !first_success {
        return Ok((false, "First testing pass failed - aborted.".to_string()));
    }

    let (success, output) = run_cargo_cmd(env, repo_env, repo_config, cargo_command, args)?;
    if success {
        if test_repo(env, repo_env, repo_config)?.0 {
            // All good
            git_commit(&repo_env.repo_path, cargo_command, commit_msg)?;
            Ok((true, output))
        } else {
            // Not all tests pass after running command
            Ok((false, output))
        }
    } else {
        // Command failed
        Ok((false, output))
    }
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
    repo_config: &RepoConfig,
    command: &str,
    args: Vec<String>,
) -> ArgosResult<(bool, String)> {
    execute_in_docker(command, args, env, repo_env, repo_config)
}

/// Executes a command in a docker container
pub fn execute_in_docker(
    cargo_command: &str,
    cargo_args: Vec<String>,
    env: &Environment,
    repo_env: &RepoEnvironment,
    repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let requires_ext = get_repo_requires_ext(repo_config, cargo_command);
    let dockerfile = find_dockerfile(cargo_command, env, repo_env, requires_ext)?;
    let image_tag = format!("argos-{}-{}", repo_env.repo, cargo_command);

    // 1. Build the image
    let build_status = std::process::Command::new("docker")
        .arg("build")
        .arg("--pull")
        .arg("-t")
        .arg(&image_tag)
        .arg("-f")
        .arg(&dockerfile)
        .arg(&repo_env.repo_path)
        .output()
        .map_err(|e| ArgosError::IntegrateRepo(format!("Failed to build docker image: {e}")))?;

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
        .ok_or_else(|| ArgosError::Environment("Invalid repo path".to_string()))?;
    let registry_path = env
        .argos_cargo_registry_path
        .to_str()
        .ok_or_else(|| ArgosError::Environment("Invalid registry cache path".to_string()))?;
    let git_path = env
        .argos_cargo_git_path
        .to_str()
        .ok_or_else(|| ArgosError::Environment("Invalid git cache path".to_string()))?;

    let mut args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "--user".to_string(),
        host_user,
        "-v".to_string(),
        format!("{}:/app", repo_path),
        "-v".to_string(),
        format!("{}:/usr/local/cargo/registry", registry_path),
        "-v".to_string(),
        format!("{}:/usr/local/cargo/git", git_path),
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
        .map_err(|e| ArgosError::IntegrateRepoTest(e.to_string()))?;

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
    requires_ext: bool,
) -> ArgosResult<std::path::PathBuf> {
    let specific = repo_env
        .repo_config_dir_path
        .join(command)
        .join("Dockerfile");
    if specific.exists() {
        Ok(specific)
    } else {
        let general = repo_env.repo_config_dir_path.join("Dockerfile");
        if general.exists() {
            Ok(general)
        } else {
            if requires_ext {
                return Err(ArgosError::IntegrateRepo(format!(
                    "Command {command} requires an external Dockerfile, but none was found in the repository."
                )));
            }
            Ok(env.default_dockerfile_path.clone())
        }
    }
}

/// Helper function to extract `requires_ext` for a command from `RepoConfig`
fn get_repo_requires_ext(repo_config: &RepoConfig, command: &str) -> bool {
    if let Some(obj) = &repo_config.cmd_requires_ext
        && let Some(requires) = obj.get(command)
        && let Some(boolean) = requires.as_boolean()
    {
        return *boolean;
    }
    false
}
