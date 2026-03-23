use std::path::Path;

use nabu::{Object, XffValue};

use crate::{
    env::RepoEnvironment,
    error::{ArgosError, ArgosResult},
};

pub struct RepoConfig {
    pub commands: Vec<String>,
    pub dependencies: Option<Vec<String>>,
    // Just a k-v store for command:[args]
    pub cmd_args: Option<Object>,
    // Just a k-v store for booleans; command:[true/false]
    pub cmd_requires_ext: Option<Object>,
}

impl RepoConfig {
    pub fn new(repo_env: &RepoEnvironment) -> ArgosResult<Self> {
        if repo_env.repo_advanced_config_path.exists() {
            let advanced_config = try_read_json(&repo_env.repo_advanced_config_path)?;
            deconstruct_advanced_config(&advanced_config)
        } else if repo_env.repo_basic_config_path.exists() {
            let basic_config = try_read_json(&repo_env.repo_basic_config_path)?;
            deconstruct_basic_config(&basic_config)
        } else {
            Ok(fallback_constructor())
        }
    }
}

fn deconstruct_basic_config(basic_config: &XffValue) -> ArgosResult<RepoConfig> {
    if let Some(array) = basic_config.as_array() {
        let mut commands = vec![];
        let all_commands = all_commands();
        for command in array {
            if let Some(command) = command.into_string() {
                let command = command.to_lowercase().trim().to_string();
                if command == "all" {
                    return Ok(fallback_constructor());
                } else if all_commands.contains(&command) {
                    commands.push(command);
                } else {
                    return Err(ArgosError::SetupRepoConfig(format!(
                        "Unknown command: {command}"
                    )));
                }
            } else {
                return Err(ArgosError::SetupRepoConfig(
                    "Basic config is not a string array".to_string(),
                ));
            }
        }
        Ok(RepoConfig {
            commands,
            dependencies: None,
            cmd_args: None,
            cmd_requires_ext: None,
        })
    } else {
        Err(ArgosError::SetupRepoConfig(
            "Basic config is not an array".to_string(),
        ))
    }
}

fn deconstruct_advanced_config(advanced_config: &XffValue) -> ArgosResult<RepoConfig> {
    if let Some(config) = advanced_config.as_object() {
        let all_commands = all_commands();
        let mut commands = vec![];
        let mut dependencies = vec![];
        let mut cmd_args = Object::new();
        let mut cmd_requires_ext = Object::new();
        let mut all = false;
        for (key, value) in config {
            // No chaining because of the compiler
            let key = key.to_lowercase();
            let key = key.as_str();
            if key == "requires" {
                let value = value.as_array().ok_or_else(|| {
                    ArgosError::SetupRepoConfig("Requires is not an array".to_string())
                })?;
                for dep in value {
                    if let Some(dependency) = dep.into_string() {
                        dependencies.push(dependency);
                    } else {
                        return Err(ArgosError::SetupRepoConfig(
                            "Dependency is not a string".to_string(),
                        ));
                    }
                }
                continue;
            }
            if key == "all" {
                all = true;
                continue;
            }
            if all_commands.contains(&key.to_owned()) {
                commands.push(key.to_string());
                if let Some(inner_data) = value.as_object() {
                    for (inner_key, inner_value) in inner_data {
                        let inner_key = inner_key.to_lowercase();
                        let inner_key = inner_key.as_str();
                        if inner_key == "args" {
                            let inner_arg_value = inner_value.as_array().ok_or_else(|| {
                                ArgosError::SetupRepoConfig("Args is not an array".to_string())
                            })?;
                            cmd_args.insert(key, XffValue::from(inner_arg_value.clone()));
                        } else if inner_key == "requires_ext" {
                            let inner_value = inner_value.as_boolean().ok_or_else(|| {
                                ArgosError::SetupRepoConfig(
                                    "Requires_ext is not a boolean".to_string(),
                                )
                            })?;
                            cmd_requires_ext.insert(key, XffValue::from(*inner_value));
                        } else {
                            return Err(ArgosError::SetupRepoConfig(format!(
                                "Unknown inner key: {inner_key}"
                            )));
                        }
                    }
                }
            } else {
                return Err(ArgosError::SetupRepoConfig(format!(
                    "Unknown command: {key}"
                )));
            }
        }
        if all {
            for command in all_commands {
                if !commands.contains(&command) {
                    commands.push(command);
                }
            }
        }
        Ok(RepoConfig {
            commands,
            dependencies: Some(dependencies),
            cmd_args: Some(cmd_args),
            cmd_requires_ext: Some(cmd_requires_ext),
        })
    } else {
        Err(ArgosError::SetupRepoConfig(
            "Advanced config is not an object".to_string(),
        ))
    }
}

/// Fallback constructor
///
/// Provides a default repo config;
/// Pretends a `argus.json` exists with "all" as contents
fn fallback_constructor() -> RepoConfig {
    RepoConfig {
        commands: all_commands(),
        dependencies: None,
        cmd_args: None,
        cmd_requires_ext: None,
    }
}

fn all_commands() -> Vec<String> {
    vec![
        "test".to_string(),
        "build".to_string(),
        "doc".to_string(),
        "doc-test".to_string(),
        "clippy".to_string(),
        "format".to_string(),
        "update".to_string(),
        "license".to_string(),
    ]
}

fn try_read_json(path: &Path) -> ArgosResult<XffValue> {
    Ok(mawu::read::json(path)?)
}
