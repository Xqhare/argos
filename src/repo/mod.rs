use crate::{env::Environment, error::ArgosResult, repo::setup::setup_repo};

mod setup;

pub fn continously_integrate_repo(env: &Environment, repo: &str) -> ArgosResult<()> {
    setup_repo(env, repo)?;
    Ok(())
}
