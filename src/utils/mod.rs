pub mod git;

use std::path::Path;

use nabu::XffValue;

use crate::{
    env::Environment,
    error::{ArgosError, ArgosResult},
    utils::git::latest_git_hash,
};
pub fn was_updated(env: &Environment, repo: &str, repo_path: &Path) -> ArgosResult<bool> {
    let latest_hash = latest_git_hash(&repo_path)?;
    let mut repo_metadata =
        match nabu::serde::read(env.argos_repo_tracking_path.join(format!("{}.xff", repo))) {
            Ok(xff) => {
                if xff.is_object() {
                    xff.into_object().unwrap()
                } else {
                    return Err(ArgosError::XffValueError(
                        "Repo metadata XFF must be an object.".to_string(),
                    ));
                }
            }
            Err(e) => return Err(ArgosError::XffError(e.to_string())),
        };
    let previous_hash = match repo_metadata.get("hash") {
        Some(x) => match x.as_string() {
            Some(x) => x,
            None => {
                return Err(ArgosError::XffValueError(
                    "Hash must be a string.".to_string(),
                ));
            }
        },
        None => {
            return Err(ArgosError::XffValueError(
                "Hash must be present.".to_string(),
            ));
        }
    };
    if &latest_hash == previous_hash {
        return Ok(false);
    }
    repo_metadata.insert("hash", latest_hash);
    nabu::serde::write(
        env.argos_repo_tracking_path.join(format!("{}.xff", repo)),
        XffValue::from(repo_metadata),
    )
    .map_err(|e| ArgosError::XffError(e.to_string()))?;
    Ok(true)
}
