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
    let license_file = match find_license_file(repo_env) {
        Ok(path) => path,
        Err(e) => return Ok((false, e.to_string())),
    };
    let license_text = match std::fs::read_to_string(&license_file) {
        Ok(text) => text,
        Err(e) => {
            return Ok((
                false,
                format!("Could not read license file: {e}"),
            ));
        }
    };
    let (license_last_year, all_text_to_last_year, all_text_after_last_year) =
        match parse_license_text(&license_text) {
            Ok(data) => data,
            Err(e) => return Ok((false, e.to_string())),
        };
    let last_commit_year = latest_git_commit_year(&repo_env.repo_path)?;
    if last_commit_year == license_last_year {
        Ok((true, "License year is already up to date".to_string()))
    } else {
        let license_text = build_license_text(
            &last_commit_year,
            &all_text_to_last_year,
            &all_text_after_last_year,
        );
        save_license_file(repo_env, &license_text)?;
        git_commit(&repo_env.repo_path, "License", "Updated license year")?;
        Ok((true, "Updated license year".to_string()))
    }
}

fn build_license_text(
    year: &str,
    all_text_to_last_year: &str,
    all_text_after_last_year: &str,
) -> String {
    let separator = if all_text_to_last_year.contains('-') && !all_text_to_last_year.contains(',') {
        "-"
    } else {
        ", "
    };
    format!("{all_text_to_last_year}{separator}{year}{all_text_after_last_year}")
}

/// Returns a tuple of (year, `all_text_to_last_year`, `all_text_after_last_year`)
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
    if !is_mit_license(license_text) {
        return Err(ArgosError::IntegrateRepoLicense(
            "License is not MIT".to_string(),
        ));
    }

    // Find "Copyright (c)" or "Copyright"
    let copyright_index = license_text
        .to_lowercase()
        .find("copyright")
        .ok_or_else(|| {
            ArgosError::IntegrateRepoLicense("Could not find 'Copyright' in license".to_string())
        })?;

    // Search for the last 4-digit year AFTER the "Copyright" mention
    let mut last_year = String::new();
    let mut last_year_end_byte_index = 0;

    let search_slice = &license_text[copyright_index..];
    let chars: Vec<char> = search_slice.chars().collect();
    let char_indices: Vec<(usize, char)> = search_slice.char_indices().collect();

    for i in 0..chars.len().saturating_sub(3) {
        if chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
        {
            last_year = chars[i..i + 4].iter().collect();
            // The byte index relative to the start of search_slice
            last_year_end_byte_index = if i + 4 < char_indices.len() {
                char_indices[i + 4].0
            } else {
                search_slice.len()
            };
        }
    }

    if last_year.is_empty() {
        return Err(ArgosError::IntegrateRepoLicense(
            "Could not find license year".to_string(),
        ));
    }

    // Adjust last_year_end_byte_index to be relative to the start of license_text
    let absolute_end_index = copyright_index + last_year_end_byte_index;

    let all_text_to_last_year = license_text[..absolute_end_index].to_string();
    let all_text_after_last_year = license_text[absolute_end_index..].to_string();

    Ok((last_year, all_text_to_last_year, all_text_after_last_year))
}

fn is_mit_license(license_text: &str) -> bool {
    license_text.contains("MIT License")
}

fn find_license_file(repo_env: &RepoEnvironment) -> ArgosResult<PathBuf> {
    let basic = repo_env.repo_path.join("LICENSE");
    let specific = repo_env.repo_path.join("LICENSE-MIT");
    if specific.exists() {
        Ok(specific)
    } else if basic.exists() {
        Ok(basic)
    } else {
        Err(ArgosError::IntegrateRepoLicense(
            "Could not find license file".to_string(),
        ))
    }
}

fn save_license_file(repo_env: &RepoEnvironment, license_text: &str) -> ArgosResult<()> {
    let license_file = find_license_file(repo_env)?;
    std::fs::write(license_file, license_text)
        .map_err(|e| ArgosError::IntegrateRepoLicense(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_license_text() {
        let text = "MIT License\n\nCopyright (c) 2022, 2024, 2025\n\nSome license text";
        let (year, to, after) = parse_license_text(text).unwrap();
        assert_eq!(year, "2025");
        assert_eq!(to, "MIT License\n\nCopyright (c) 2022, 2024, 2025");
        assert_eq!(after, "\n\nSome license text");
    }

    #[test]
    fn test_parse_license_text_hyphen() {
        let text = "MIT License\n\nCopyright (c) 2022-2025\n\nSome license text";
        let (year, to, after) = parse_license_text(text).unwrap();
        assert_eq!(year, "2025");
        assert_eq!(to, "MIT License\n\nCopyright (c) 2022-2025");
        assert_eq!(after, "\n\nSome license text");
    }
}
