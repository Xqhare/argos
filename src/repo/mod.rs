use crate::{
    env::{Environment, RepoEnvironment},
    repo::{
        integrate::{integrate_repo, save_failed_integration},
        setup::setup_repo,
    },
    utils::read_all_succeeded,
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
        Ok(was_updated) => {
            if !was_updated {
                match read_all_succeeded(&repo_env) {
                    Ok(b) => return b,
                    Err(e) => {
                        let _ = save_failed_integration(&repo_env, &e.to_string());
                        return false;
                    }
                }
            }
        }
        Err(e) => {
            let _ = save_failed_integration(&repo_env, &e.to_string());
            return false;
        }
    };
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

    true
}
