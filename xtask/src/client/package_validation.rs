// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Result, anyhow};
use std::{fs, path::Path};

pub(crate) fn get_client_version() -> Result<String> {
    let cargo_toml = fs::read_to_string("node/Cargo.toml")?;
    let mut in_package_section = false;

    for line in cargo_toml.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_package_section = trimmed == "[package]";
            continue;
        }

        if in_package_section && trimmed.starts_with("version = ") {
            return trimmed
                .split('"')
                .nth(1)
                .map(ToOwned::to_owned)
                .ok_or_else(|| anyhow!("Invalid version format in node/Cargo.toml"));
        }
    }

    Err(anyhow!(
        "Failed to read client version from the [package] section in node/Cargo.toml"
    ))
}

pub(crate) fn find_local_package_files(client_version: &str) -> Result<Vec<(String, String)>> {
    let mut packages = Vec::new();
    let mut stale_packages = Vec::new();

    collect_package_files(
        Path::new("target/debian"),
        client_version,
        &mut packages,
        &mut stale_packages,
    )?;
    collect_package_files(
        Path::new("target/generate-rpm"),
        client_version,
        &mut packages,
        &mut stale_packages,
    )?;

    reject_stale_packages(client_version, &stale_packages)?;

    Ok(packages)
}

pub(crate) fn validate_package_file_names<'a>(
    client_version: &str,
    file_names: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    let stale_packages: Vec<_> = file_names
        .into_iter()
        .filter(|file_name| is_package_file(file_name))
        .filter(|file_name| !package_file_matches_client_version(file_name, client_version))
        .map(ToOwned::to_owned)
        .collect();

    reject_stale_packages(client_version, &stale_packages)
}

fn collect_package_files(
    directory: &Path,
    client_version: &str,
    packages: &mut Vec<(String, String)>,
    stale_packages: &mut Vec<String>,
) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !is_package_file(file_name) {
            continue;
        }

        if package_file_matches_client_version(file_name, client_version) {
            packages.push((file_name.to_string(), path.to_string_lossy().to_string()));
        } else {
            stale_packages.push(path.to_string_lossy().to_string());
        }
    }

    Ok(())
}

fn reject_stale_packages(client_version: &str, stale_packages: &[String]) -> Result<()> {
    if stale_packages.is_empty() {
        return Ok(());
    }

    Err(anyhow!(
        "Refusing to upload stale package artifacts for client version {}.\n\
         Remove the stale files before retrying:\n{}",
        client_version,
        stale_packages
            .iter()
            .map(|path| format!("  - {path}"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn is_package_file(file_name: &str) -> bool {
    file_name.ends_with(".deb") || file_name.ends_with(".rpm")
}

fn package_file_matches_client_version(file_name: &str, client_version: &str) -> bool {
    if file_name.ends_with(".deb") {
        file_name.starts_with("duniter_") && file_name.contains(&format!("_{client_version}-"))
    } else if file_name.ends_with(".rpm") {
        file_name.starts_with("duniter-") && file_name.contains(&format!("-{client_version}-"))
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{package_file_matches_client_version, validate_package_file_names};

    #[test]
    fn accepts_matching_deb_and_rpm_names() {
        assert!(package_file_matches_client_version(
            "duniter_2.0.0-1_amd64.deb",
            "2.0.0"
        ));
        assert!(package_file_matches_client_version(
            "duniter-2.0.0-1.x86_64.rpm",
            "2.0.0"
        ));
    }

    #[test]
    fn rejects_stale_package_names() {
        assert!(!package_file_matches_client_version(
            "duniter_0.12.0-1_amd64.deb",
            "2.0.0"
        ));
        assert!(!package_file_matches_client_version(
            "duniter-0.14.0-1.x86_64.rpm",
            "2.0.0"
        ));
    }

    #[test]
    fn validation_reports_stale_packages() {
        let error = validate_package_file_names(
            "2.0.0",
            ["duniter_2.0.0-1_amd64.deb", "duniter_0.12.0-1_amd64.deb"],
        )
        .unwrap_err();

        assert!(error.to_string().contains("duniter_0.12.0-1_amd64.deb"));
    }
}
