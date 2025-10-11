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
use std::{fs, io::Write, path::Path};

/// Download artifacts from a successful job
/// # Arguments
/// * `gitlab_token` - The GitLab authentication token
/// * `project_id` - The GitLab project ID (URL-encoded)
/// * `job_id` - The job ID to download artifacts from
/// * `output_path` - Local path where to save the artifacts (will be a zip file)
pub async fn download_job_artifacts(
    gitlab_token: String,
    project_id: String,
    job_id: u64,
    output_path: &Path,
) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "https://git.duniter.org/api/v4/projects/{}/jobs/{}/artifacts",
            project_id, job_id
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .send()
        .await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(output_path)?;
        file.write_all(&bytes)?;

        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(anyhow!(
            "Failed to download artifacts (status {}): {}",
            status,
            error_text
        ))
    }
}

/// Extract a zip archive to a directory
/// # Arguments
/// * `zip_path` - Path to the zip file
/// * `output_dir` - Directory where to extract files
pub fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = output_dir.join(file.name());

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
