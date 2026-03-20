use nabu::Array;

use crate::{
    env::Environment,
    error::{ArgosError, ArgosResult},
    repo::continously_integrate_repo,
};

mod env;
mod error;
mod repo;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up and check environment
    let mut env = Environment::new()?;
    // Continously integrate
    continously_integrate(&mut env)?;
    Ok(())
}

fn continously_integrate(env: &mut Environment) -> ArgosResult<()> {
    let repo_list = read_repo_list(env)?;
    let mut failed_repos: Vec<String> = Vec::new();
    for repo in repo_list {
        let repo = repo.into_string().ok_or(ArgosError::XffValueError(
            "Repo array must contain only strings as children.".to_string(),
        ))?;
        let ci_repo = continously_integrate_repo(&env, &repo, &failed_repos);
        if ci_repo.is_ok_and(|v| v) {
            continue;
        } else {
            failed_repos.push(repo);
        }
    }
    Ok(())
}

fn read_repo_list(env: &mut Environment) -> ArgosResult<Array> {
    let repo_list = mawu::read::json(&env.repo_list_path)?.into_object();
    if let Some(repo_list) = repo_list {
        // Providing a git_root is optional
        if let Some(git_root) = repo_list.get("git_root") {
            env.git_root_url = git_root.into_string().ok_or(ArgosError::XffValueError(
                "Supplied git root must be a string".to_string(),
            ))?;
        };
        match repo_list.get("repos") {
            Some(repos) => match repos.into_array() {
                Some(repos) => Ok(repos),
                None => Err(ArgosError::JsonError(
                    "Repo list is not an array".to_string(),
                )),
            },
            None => Err(ArgosError::JsonError(
                "Repo list is missing repos array".to_string(),
            )),
        }
    } else {
        Err(ArgosError::JsonError(
            "Repo list is not an array".to_string(),
        ))
    }
}
