use std::path::PathBuf;

use crate::{
    env::RepoEnvironment,
    error::{ArgosError, ArgosResult},
    repo::config::RepoConfig,
    utils::git::{git_commit, latest_git_commit_year},
};

/// Updates the year inside the license file.
/// Only supports MIT licenses. It searches for either `LICENSE` or `LICENSE-MIT` files, prefering the
/// latter.
///
/// To validate, it only checks if the license file starts with `MIT License`.
///
/// Only updates the year if a commit was made during it.
///
/// It is very simple:
///
/// 1. Find the LAST 4 character number
/// 2. Insert ", {year}" after that number
/// 3. Save
///
/// # Notes
///
/// The parser supports these kind of listings:
///
/// `2022, 2024, 2025`
/// `2022, 2024-2025`
///
/// Adding a new year (e.g. 2026) would look like:
///
/// `2022, 2024, 2025, 2026`
/// `2022, 2024-2025, 2026`
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config
///
/// # Returns
/// Returns a boolean indicating if the license was successful and a string containing the output
pub fn license_repo(
    repo_env: &RepoEnvironment,
    _repo_config: &RepoConfig,
) -> ArgosResult<(bool, String)> {
    let license_file = find_license_file(repo_env)?;
    let license_text = match std::fs::read_to_string(&license_file) {
        Ok(text) => text,
        Err(_) => {
            return Err(ArgosError::IntegrateRepoLicenseError(
                "Could not read license file".to_string(),
            ));
        }
    };
    let (license_last_year, all_text_to_last_year, all_text_after_last_year) =
        parse_license_text(&license_text)?;
    let last_commit_year = latest_git_commit_year(&repo_env.repo_path)?;
    if last_commit_year != license_last_year {
        let license_text = build_license_text(
            &last_commit_year,
            &all_text_to_last_year,
            &all_text_after_last_year,
        );
        save_license_file(repo_env, &license_text)?;
        git_commit(&repo_env.repo_path, "License", "Updated license year")?;
    }

    Ok((true, "Updated license year".to_string()))
}

fn build_license_text(
    year: &str,
    all_text_to_last_year: &str,
    all_text_after_last_year: &str,
) -> String {
    format!(
        "{}, {} {}",
        all_text_to_last_year, year, all_text_after_last_year
    )
}

/// Returns a tuple of (year, all_text_to_last_year, all_text_after_last_year)
///
/// E.g
///
/// ```text
/// MIT License
///
/// Copyright (c) 2022, 2024, 2025
///
/// ...
/// ```
/// it will return
/// ```
/// ("2025", "Copyright (c) 2022, 2024, 2025", "...")
/// ```
///
/// trailing commas are ignored
fn parse_license_text(license_text: &str) -> ArgosResult<(String, String, String)> {
    if is_mit_license(license_text) {
        let mut found_pot_year = false;
        let mut pot_year = String::new();
        let mut length = 0;
        for char in license_text.chars().rev() {
            if found_pot_year && pot_year.len() == 4 {
                if char == ',' || char == ' ' || char == '-' {
                    break;
                } else {
                    found_pot_year = false;
                }
            }
            if char.is_digit(10) && !found_pot_year {
                found_pot_year = true;
                pot_year.push(char);
            } else if char.is_digit(10) && found_pot_year && pot_year.len() < 4 {
                pot_year.push(char);
            } else if char.is_digit(10) && found_pot_year && pot_year.len() == 4 {
                found_pot_year = false;
            }
            length += 1;
        }
        if !found_pot_year {
            return Err(ArgosError::IntegrateRepoLicenseError(
                "Could not find license year".to_string(),
            ));
        }
        let all_text_to_last_year = license_text[..length].to_string();
        let all_text_after_last_year = license_text[length..].to_string();
        Ok((pot_year, all_text_to_last_year, all_text_after_last_year))
    } else {
        Err(ArgosError::IntegrateRepoLicenseError(
            "License is not MIT".to_string(),
        ))
    }
}

fn is_mit_license(license_text: &str) -> bool {
    if license_text.starts_with("MIT License") {
        true
    } else {
        false
    }
}

fn find_license_file(repo_env: &RepoEnvironment) -> ArgosResult<PathBuf> {
    let basic = repo_env.repo_path.join("LICENSE");
    let specific = repo_env.repo_path.join("LICENSE-MIT");
    if specific.exists() {
        Ok(specific)
    } else if basic.exists() {
        Ok(basic)
    } else {
        Err(ArgosError::IntegrateRepoLicenseError(
            "Could not find license file".to_string(),
        ))
    }
}

fn save_license_file(repo_env: &RepoEnvironment, license_text: &str) -> ArgosResult<()> {
    let license_file = find_license_file(repo_env)?;
    std::fs::write(license_file, license_text)
        .map_err(|e| ArgosError::IntegrateRepoLicenseError(e.to_string()))?;
    Ok(())
}
