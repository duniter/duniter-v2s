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
use std::{path::Path, process::Command};

/// Extract runtime name from network name (e.g., "g1-1000" -> "g1")
pub fn extract_runtime(network: &str) -> Result<&str> {
    if network.starts_with("g1") {
        Ok("g1")
    } else if network.starts_with("gdev") {
        Ok("gdev")
    } else if network.starts_with("gtest") {
        Ok("gtest")
    } else {
        Err(anyhow!(
            "Unknown network: {}. Supported networks are g1-*, gdev-*, and gtest-*.",
            network
        ))
    }
}

/// Ensures that `node/specs/{runtime}-raw.json` exists before compilation.
///
/// This file is required at compile time by `include_bytes!` when building with the `embed` feature.
/// It is NOT versioned in git (gitignored) to avoid bloating the repository.
///
/// Resolution order:
/// 1. If the file already exists locally (e.g., after running `build-raw-specs`), do nothing.
/// 2. If `RAW_SPEC_URL` env var is set (provided by `trigger-builds` in CI), download from that URL.
/// 3. Otherwise, fail with a helpful error message.
pub fn ensure_raw_spec(network: &str) -> Result<()> {
    let runtime = extract_runtime(network)?;
    let raw_spec_path = format!("node/specs/{}-raw.json", runtime);

    if Path::new(&raw_spec_path).exists() {
        println!("   Raw spec already exists: {}", raw_spec_path);
        return Ok(());
    }

    // Try downloading from RAW_SPEC_URL (set by trigger_release_builds in CI)
    if let Ok(url) = std::env::var("RAW_SPEC_URL") {
        println!("   Downloading {}-raw.json from release...", runtime);

        std::fs::create_dir_all("node/specs/")?;

        let status = Command::new("curl")
            .args([
                "-fSL",
                "--retry",
                "3",
                "--retry-delay",
                "5",
                "-o",
                &raw_spec_path,
                &url,
            ])
            .status()
            .map_err(|e| anyhow!("Failed to execute curl: {}. Is curl installed?", e))?;

        if !status.success() {
            // Clean up partial download
            let _ = std::fs::remove_file(&raw_spec_path);
            return Err(anyhow!(
                "Failed to download raw spec from {}.\n\
                 curl exited with status: {}",
                url,
                status
            ));
        }

        // Verify the file was downloaded and is not empty
        let metadata = std::fs::metadata(&raw_spec_path)
            .map_err(|_| anyhow!("Downloaded file not found: {}", raw_spec_path))?;
        if metadata.len() == 0 {
            let _ = std::fs::remove_file(&raw_spec_path);
            return Err(anyhow!("Downloaded file is empty: {}", raw_spec_path));
        }

        println!(
            "   Downloaded {} ({:.1} MB)",
            raw_spec_path,
            metadata.len() as f64 / 1_048_576.0
        );
        return Ok(());
    }

    Err(anyhow!(
        "Raw spec file not found: {}\n\
         \n\
         This file is required for compilation with the 'embed' feature.\n\
         \n\
         To generate it locally:\n\
         \n\
         \x20 cargo xtask release client build-raw-specs {}\n\
         \n\
         In CI, this file is automatically downloaded from the GitLab release\n\
         via the RAW_SPEC_URL variable set by trigger-builds.",
        raw_spec_path,
        network
    ))
}
