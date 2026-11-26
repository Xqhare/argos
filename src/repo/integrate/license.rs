use std::path::PathBuf;

use thoth::Thoth;

use crate::{
    env::RepoEnvironment,
    error::{ArgosError, ArgosResult},
    repo::config::RepoConfig,
    utils::git::latest_git_commit_year,
};

/// Updates the year inside the license file.
/// Only supports MIT licenses. It searches for either `LICENSE` or `LICENSE-MIT` files, preferring the latter.
///
/// To validate, it only checks if the license file starts with `MIT License`.
///
/// Only updates the year if a commit was made during it.
///
/// The generated text defaults to using '-' to mark continuous years of copyright, and ',' if more than one year without commits is observed.
///
/// For example, if the current year is '2026':
///
/// "MIT License\n\nCopyright (c) 2022-2024\n\nSome license text"
/// Transforms into:
/// "MIT License\n\nCopyright (c) 2022-2024, 2026\n\nSome license text"
///
///
/// "MIT License\n\nCopyright (c) 2021-2023, 2025\n\nSome license text"
/// Transforms into:
/// "MIT License\n\nCopyright (c) 2021-2023, 2025-2026\n\nSome license text"
///
/// "MIT License\n\nCopyright (c) 2022-2025\n\nSome license text"
/// Transforms into:
/// "MIT License\n\nCopyright (c) 2022-2026\n\nSome license text"
///
/// # Arguments
/// * `repo_env` - Repo environment
/// * `repo_config` - Repo config (unused; For unified function calling in 'integrate/mod.rs')
///
/// # Returns
/// Returns a boolean indicating if the license was successful and a string containing the output
///
/// # Notes
///
/// Assumes years are *always* 4 characters long and only ',' & '-' are used for year separation.
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
            return Ok((false, format!("Could not read license file: {e}")));
        }
    };
    let last_commit_year = latest_git_commit_year(&repo_env.repo_path)?;
    handle_license_text(&license_text, &last_commit_year, repo_env, false)
}

fn handle_license_text(
    license_text: &str,
    last_commit_year: &str,
    repo_env: &RepoEnvironment,
    testing_mode: bool,
) -> ArgosResult<(bool, String)> {
    if !is_mit_license(license_text) {
        return Ok((true, "License is not MIT - skipping update".to_string()));
    }
    let (last_license_year, all_text_to_last_year, all_text_after_last_year) =
        match parse_license_text(license_text) {
            Ok(data) => data,
            Err(e) => return Ok((false, e.to_string())),
        };

    let last_commit_year_u16: u16 = last_commit_year.parse().map_err(|err| {
        ArgosError::IntegrateRepoLicense(format!(
            "Unable to cast year '{last_commit_year}' into a valid u16 integer!\nError: {err}"
        ))
    })?;
    let last_license_year_u16: u16 = last_license_year.parse().map_err(|err| {
        ArgosError::IntegrateRepoLicense(format!(
            "Unable to cast year '{last_license_year}' into a valid u16 integer!\nError: {err}"
        ))
    })?;
    if last_commit_year_u16 <= last_license_year_u16 {
        Ok((true, "License year is already up to date".to_string()))
    } else {
        let license_text = build_license_text(
            last_commit_year_u16,
            last_license_year_u16,
            &all_text_to_last_year,
            &all_text_after_last_year,
        )?;
        if testing_mode {
            Ok((true, license_text))
        } else {
            save_license_file(repo_env, &license_text)?;
            Ok((true, "Updated license year".to_string()))
        }
    }
}

fn build_license_text(
    last_commit_year_u16: u16,
    last_license_year_u16: u16,
    all_text_to_last_year: &str,
    all_text_after_last_year: &str,
) -> ArgosResult<String> {
    let year_delta = { last_commit_year_u16.saturating_sub(last_license_year_u16) };
    // NOTE: If every bit of performance is needed, load thoth once and reuse the utf tables instead of reloading them on every build_license call && run on the read &[u8] directly
    let graphemes = Thoth::grapheme_segmentation(all_text_to_last_year).map_err(|err| {
        ArgosError::Environment(format!("Thoth cannot segment text into graphemes: '{err}'"))
    })?;
    let previous_hypen_con = {
        // Assuming 4 # long years only
        match graphemes.get(graphemes.len().saturating_sub(5)).ok_or({
            ArgosError::IntegrateRepoLicense(format!(
                "'build_license_text' encountered string with fewer items then its own length - 5"
            ))
        })?.as_str() {
            "-" => true,
            _ => false,
        }
    };
    if year_delta == 0 {
        debug_assert!(
            false,
            "'build_license_text' called with new_year already present in file. This must be cought by the caller prior to calling for correct reporting."
        );
        return Ok(format!("{all_text_to_last_year}{all_text_after_last_year}"));
    }
    let separator = if year_delta == 1 { "-" } else { ", " };
    let all_text_to_last_year = if previous_hypen_con && year_delta == 1 {
        let mutated_text: Vec<String> = graphemes[..graphemes.len().saturating_sub(5)]
            .iter()
            .cloned()
            .collect();
        &String::from_iter(mutated_text)
    } else {
        all_text_to_last_year
    };
    Ok(format!(
        "{all_text_to_last_year}{separator}{last_commit_year_u16}{all_text_after_last_year}"
    ))
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
    fn dummy_repo_env() -> RepoEnvironment {
        RepoEnvironment {
            repo: Default::default(),
            repo_git_url: Default::default(),
            repo_path: Default::default(),
            repo_tracking_xff: Default::default(),
            repo_tracking_json: Default::default(),
            repo_history_dir: Default::default(),
            repo_config_dir_path: Default::default(),
            repo_basic_config_path: Default::default(),
            repo_advanced_config_path: Default::default(),
        }
    }

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

    #[test]
    fn full_text_handling_one_year_non_seq() {
        let text = "MIT License\n\nCopyright (c) 2024\n\nSome license text";
        let new_txt = handle_license_text(text, "2026", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text = "MIT License\n\nCopyright (c) 2024, 2026\n\nSome license text";
        assert_eq!(new_txt, expected_text)
    }
    #[test]
    fn full_text_handling_one_year_seq() {
        let text = "MIT License\n\nCopyright (c) 2025\n\nSome license text";
        let new_txt = handle_license_text(text, "2026", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text = "MIT License\n\nCopyright (c) 2025-2026\n\nSome license text";
        assert_eq!(new_txt, expected_text)
    }
    #[test]
    fn full_text_handling_up_to_date() {
        let text = "MIT License\n\nCopyright (c) 2022-2025\n\nSome license text";
        let new_txt = handle_license_text(text, "2025", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text = "License year is already up to date";
        assert_eq!(new_txt, expected_text)
    }
    #[test]
    fn full_text_handling_hyphen() {
        let text = "MIT License\n\nCopyright (c) 2022-2025\n\nSome license text";
        let new_txt = handle_license_text(text, "2026", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text = "MIT License\n\nCopyright (c) 2022-2026\n\nSome license text";
        assert_eq!(new_txt, expected_text)
    }
    #[test]
    fn full_text_handling_hyphen_non_sequential() {
        let text = "MIT License\n\nCopyright (c) 2022-2024\n\nSome license text";
        let new_txt = handle_license_text(text, "2026", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text = "MIT License\n\nCopyright (c) 2022-2024, 2026\n\nSome license text";
        assert_eq!(new_txt, expected_text)
    }
    #[test]
    fn full_text_handling_hyphen_non_sequential_continuation() {
        let text = "MIT License\n\nCopyright (c) 2022-2024, 2026\n\nSome license text";
        let new_txt = handle_license_text(text, "2027", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text =
            "MIT License\n\nCopyright (c) 2022-2024, 2026-2027\n\nSome license text";
        assert_eq!(new_txt, expected_text)
    }
    #[test]
    fn full_text_handling_commas() {
        let text = "MIT License\n\nCopyright (c) 2022, 2024, 2025\n\nSome license text";
        let new_txt = handle_license_text(text, "2026", &dummy_repo_env(), true);
        assert!(new_txt.is_ok());
        let new_txt = new_txt.unwrap().1;
        let expected_text =
            "MIT License\n\nCopyright (c) 2022, 2024, 2025-2026\n\nSome license text";
        assert_eq!(new_txt, expected_text)
    }
}
