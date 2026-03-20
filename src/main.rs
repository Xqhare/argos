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
    let env = Environment::new()?;
    // Continously integrate
    continously_integrate(&env)?;
    Ok(())
}

fn continously_integrate(env: &Environment) -> ArgosResult<()> {
    let repo_list = read_repo_list(&env)?;
    for repo in repo_list {
        continously_integrate_repo(
            &env,
            &repo.into_string().ok_or(ArgosError::XffValueError(
                "Repo array must contain only strings as children.".to_string(),
            ))?,
        )?;
    }
    Ok(())
}

fn read_repo_list(env: &Environment) -> ArgosResult<Array> {
    let repo_list = mawu::read::json(&env.repo_list_path)?.into_array();
    if let Some(repo_list) = repo_list {
        Ok(repo_list)
    } else {
        Err(ArgosError::JsonError(
            "Repo list is not an array".to_string(),
        ))
    }
}
