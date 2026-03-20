use std::path::Path;

use nabu::{Object, XffValue};

use crate::{
    env::{Environment, RepoEnvironment},
    error::{ArgosError, ArgosResult},
};

pub struct RepoConfig {
    pub commands: Vec<String>,
    pub dependencies: Option<Vec<String>>,
    // Just a k-v store for command:[args]
    pub cmd_args: Option<Object>,
    // Just a k-v store for booleans; command:[true/false]
    pub cmd_rquires_ext: Option<Object>,
}

impl RepoConfig {
    pub fn new(repo_env: &RepoEnvironment) -> ArgosResult<Self> {
        if !repo_env.repo_config_dir_path.exists() {
            return Ok(fallback_constructor());
        };
        if let Ok(advanced_config) = try_read_json(&repo_env.repo_advanced_config_path) {
            deconstruct_advanced_config(advanced_config)
        } else {
            if let Ok(basic_config) = try_read_json(&repo_env.repo_basic_config_path) {
                deconstruct_basic_config(basic_config)
            } else {
                Ok(fallback_constructor())
            }
        }
    }
}

fn deconstruct_basic_config(basic_config: XffValue) -> ArgosResult<RepoConfig> {}

fn deconstruct_advanced_config(advanced_config: XffValue) -> ArgosResult<RepoConfig> {}

/// Fallback constructor
///
/// Provides a default repo config;
/// Pretends a `argus.json` exists with "all" as contents
fn fallback_constructor() -> RepoConfig {
    RepoConfig {
        commands: vec![
            "test".to_string(),
            "build".to_string(),
            "doc-test".to_string(),
            "clippy".to_string(),
            "format".to_string(),
            "update".to_string(),
        ],
        dependencies: None,
        cmd_args: None,
        cmd_rquires_ext: None,
    }
}

fn try_read_json(path: &Path) -> ArgosResult<XffValue> {
    Ok(mawu::read::json(path)?)
}
