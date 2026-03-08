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
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

const PROJECT_ID: &str = "nodes%2Frust%2Fduniter-v2s";

struct JobConfig {
    name: &'static str,
    artifact_patterns: &'static [&'static str],
}

const RELEASE_JOBS: &[JobConfig] = &[
    JobConfig {
        name: "release_debian_arm",
        artifact_patterns: &["target/debian/*.deb"],
    },
    JobConfig {
        name: "release_debian_x64",
        artifact_patterns: &["target/debian/*.deb"],
    },
    JobConfig {
        name: "release_rpm_arm",
        artifact_patterns: &["target/generate-rpm/*.rpm"],
    },
    JobConfig {
        name: "release_rpm_x64",
        artifact_patterns: &["target/generate-rpm/*.rpm"],
    },
];

pub async fn finalize_release(
    network: String,
    pipeline_id: u64,
    branch: Option<String>,
    release_tag: Option<String>,
) -> Result<()> {
    println!("🚀 Finalizing client release for network: {network}");
    println!("   Pipeline ID: {pipeline_id}");

    let client_version = super::package_validation::get_client_version()?;
    let release_tag = release_tag.unwrap_or_else(|| format!("{network}-{client_version}"));
    let branch = branch.unwrap_or_else(|| format!("v{client_version}"));
    let runtime = super::ensure_raw_spec::extract_runtime(&network)?.to_string();
    let milestone = format!("client-{client_version}");

    println!("   Release tag: {release_tag}");
    println!("   Branch/ref: {branch}");
    println!("   Runtime: {runtime}");

    println!("\n🔧 Step 1: Preparing release assets without rebuilding the node...");
    prepare_release_assets(&runtime)?;

    println!("\n📥 Step 2: Downloading package artifacts from pipeline...");
    let package_artifacts = download_pipeline_package_artifacts(pipeline_id).await?;

    if package_artifacts.is_empty() {
        return Err(anyhow!(
            "No DEB/RPM artifacts were downloaded from pipeline {pipeline_id}."
        ));
    }

    let package_names: Vec<_> = package_artifacts
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .collect();
    super::package_validation::validate_package_file_names(&client_version, package_names)?;

    println!("\n🌐 Step 3: Ensuring GitLab release exists...");
    let existing_assets = match crate::gitlab::get_release_assets(release_tag.clone()).await {
        Ok(assets) => {
            println!("   ✅ Release '{release_tag}' already exists");
            assets
        }
        Err(_) => {
            println!("   Creating release '{release_tag}'...");
            crate::gitlab::release_client(release_tag.clone(), branch, milestone).await?;
            crate::gitlab::get_release_assets(release_tag.clone())
                .await
                .unwrap_or_default()
        }
    };
    let existing_asset_names: HashSet<_> =
        existing_assets.into_iter().map(|(name, _)| name).collect();

    println!("\n📤 Step 4: Uploading missing assets to the release...");
    let mut release_assets = vec![(
        format!("{runtime}_client-specs.yaml"),
        PathBuf::from(format!("release/client/{runtime}_client-specs.yaml")),
    )];

    for path in package_artifacts {
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid artifact path: {}", path.display()))?
            .to_string_lossy()
            .to_string();
        release_assets.push((file_name, path));
    }

    upload_missing_assets(&release_tag, &existing_asset_names, &release_assets).await?;

    println!("\n✅ Release finalization completed successfully!");
    println!("📋 Summary:");
    println!("   - Network: {network}");
    println!("   - Pipeline ID: {pipeline_id}");
    println!("   - Release tag: {release_tag}");
    println!("   - Uploaded assets considered: {}", release_assets.len());

    Ok(())
}

fn prepare_release_assets(runtime: &str) -> Result<()> {
    std::fs::create_dir_all("release/client/")?;

    let client_specs_src = PathBuf::from(format!("node/specs/{runtime}_client-specs.yaml"));
    let client_specs_dst = PathBuf::from(format!("release/client/{runtime}_client-specs.yaml"));

    if !client_specs_src.exists() {
        return Err(anyhow!(
            "Client specs file not found: {}",
            client_specs_src.display()
        ));
    }

    std::fs::copy(&client_specs_src, &client_specs_dst)?;
    println!("   Copied {}", client_specs_dst.display());

    Ok(())
}

async fn download_pipeline_package_artifacts(pipeline_id: u64) -> Result<Vec<PathBuf>> {
    let jobs = crate::gitlab::get_pipeline_jobs(PROJECT_ID.to_string(), pipeline_id).await?;
    let artifacts_dir = PathBuf::from("target/release-artifacts");
    std::fs::create_dir_all(&artifacts_dir)?;

    let mut downloaded = Vec::new();

    for config in RELEASE_JOBS {
        let Some(job) = jobs.iter().find(|job| job.name == config.name) else {
            println!("   ⏭️  Job not found in pipeline: {}", config.name);
            continue;
        };

        if job.status != "success" {
            println!("   ⏭️  Skipping {} (status: {})", config.name, job.status);
            continue;
        }

        println!(
            "   Downloading artifacts from {} (job ID: {})",
            config.name, job.id
        );
        let zip_path = artifacts_dir.join(format!("{}.zip", config.name));
        let extract_dir = artifacts_dir.join(config.name);

        if extract_dir.exists() {
            std::fs::remove_dir_all(&extract_dir)?;
        }

        crate::gitlab::download_job_artifacts(PROJECT_ID.to_string(), job.id, &zip_path).await?;
        crate::gitlab::extract_zip(&zip_path, &extract_dir)?;

        let mut files = Vec::new();
        find_artifact_files(&extract_dir, &mut files)?;

        for path in files {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if file_name.ends_with(".deb") || file_name.ends_with(".rpm") {
                downloaded.push(path);
            }
        }

        if config.artifact_patterns.is_empty() {
            continue;
        }
    }

    println!("   Downloaded {} package artifact(s)", downloaded.len());
    Ok(downloaded)
}

fn find_artifact_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_artifact_files(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    Ok(())
}

async fn upload_missing_assets(
    release_tag: &str,
    existing_asset_names: &HashSet<String>,
    assets: &[(String, PathBuf)],
) -> Result<()> {
    for (asset_name, path) in assets {
        if existing_asset_names.contains(asset_name) {
            println!("   ⏭️  Asset already exists on release: {asset_name}");
            continue;
        }

        if !path.exists() {
            return Err(anyhow!("Asset file does not exist: {}", path.display()));
        }

        println!("   Uploading: {asset_name}");
        let asset_url =
            crate::gitlab::upload_file(PROJECT_ID.to_string(), path, asset_name.clone()).await?;
        crate::gitlab::create_asset_link(release_tag.to_string(), asset_name.clone(), asset_url)
            .await?;
    }

    Ok(())
}
