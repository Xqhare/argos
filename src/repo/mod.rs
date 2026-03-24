use crate::{
    env::{Environment, RepoEnvironment},
    repo::{
        integrate::{integrate_repo, save_failed_integration},
        setup::setup_repo,
    },
};

mod config;
mod integrate;
mod setup;

/// Continously integrates a repo
///
/// Returns false if the integration of a repo failed
pub fn continuously_integrate_repo(env: &Environment, repo: &str, failed_repos: &[String]) -> bool {
    let repo_env = match RepoEnvironment::new(repo, env) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create repo environment for {repo}: {e}");
            return false;
        }
    };
    match setup_repo(&repo_env) {
        Ok(updated) => {
            if !updated {
                return true;
            }
        }
        Err(e) => {
            let _ = save_failed_integration(&repo_env, &e.to_string());
            return false;
        }
    }
    let repo_config = match config::RepoConfig::new(&repo_env) {
        Ok(c) => c,
        Err(e) => {
            let _ = save_failed_integration(&repo_env, &e.to_string());
            return false;
        }
    };
    match integrate_repo(env, &repo_env, &repo_config, failed_repos) {
        Ok(success) => {
            if !success {
                return false;
            }
        }
        Err(e) => {
            let _ = save_failed_integration(&repo_env, &e.to_string());
            return false;
        }
    }

    // Sleep for 1 minute - overkill, but if any other process wants I/O I yield to
    // them here
    std::thread::sleep(std::time::Duration::from_secs(60));
    true
}
