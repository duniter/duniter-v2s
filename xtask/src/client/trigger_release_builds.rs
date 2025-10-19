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
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

const PROJECT_ID: &str = "nodes%2Frust%2Fduniter-v2s";
const POLL_INTERVAL_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;

/// Job configuration
#[derive(Debug, Clone)]
struct JobConfig {
    name: String,
    #[allow(dead_code)]
    artifact_patterns: Vec<String>,
}

impl JobConfig {
    fn new(name: &str, patterns: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            artifact_patterns: patterns.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Result of a job execution
#[derive(Debug)]
struct JobResult {
    job_name: String,
    job_id: u64,
    status: String,
    artifact_patterns: Vec<String>,
}

/// Triggers release builds on GitLab CI and uploads artifacts to release
/// # Arguments
/// * `network` - The network name (e.g., gtest-1000, g1-1000, gdev-1000)
/// * `branch` - The Git branch to use
/// * `release_tag` - Optional release tag. If not provided, will be computed from network and client version
pub async fn trigger_release_builds(
    network: String,
    branch: String,
    release_tag: Option<String>,
) -> Result<()> {
    println!("🚀 Starting release build process for network: {}", network);
    println!("   Branch: {}", branch);

    // Compute release tag if not provided
    let release_tag = if let Some(tag) = release_tag {
        println!("   Release tag: {} (provided)", tag);
        tag
    } else {
        let computed_tag = compute_release_tag(&network)?;
        println!("   Release tag: {} (computed)", computed_tag);
        computed_tag
    };

    // Step 0: Check that the release exists
    println!("\n🔍 Step 0: Checking GitLab release...");
    let existing_assets = match crate::gitlab::get_release_assets(release_tag.clone()).await {
        Ok(assets) => {
            println!("   ✅ Release '{}' found", release_tag);
            println!("   Found {} existing assets", assets.len());
            if !assets.is_empty() {
                for (name, _) in &assets {
                    println!("     - {}", name);
                }
            }
            assets
        }
        Err(e) => {
            return Err(anyhow!(
                "❌ GitLab release '{}' does not exist!\n\
                \n\
                You must create the release first using:\n\
                  cargo xtask release client create {} {}\n\
                \n\
                Error details: {}",
                release_tag,
                network,
                branch,
                e
            ));
        }
    };

    // Define all possible jobs
    let all_jobs = vec![
        JobConfig::new("release_debian_arm", vec!["target/debian/*.deb"]),
        JobConfig::new("release_debian_x64", vec!["target/debian/*.deb"]),
        JobConfig::new("release_rpm_arm", vec!["target/generate-rpm/*.rpm"]),
        JobConfig::new("release_rpm_x64", vec!["target/generate-rpm/*.rpm"]),
        JobConfig::new("release_docker_arm", vec![]),
        JobConfig::new("release_docker_x64", vec![]),
        JobConfig::new("release_docker_manifest", vec![]), // Creates multi-arch manifest
    ];

    // Filter jobs based on existing assets
    let jobs: Vec<_> = all_jobs
        .into_iter()
        .filter(|job| {
            let job_has_assets = match job.name.as_str() {
                "release_debian_arm" => existing_assets
                    .iter()
                    .any(|(name, _)| name.ends_with(".deb") && name.contains("arm64")),
                "release_debian_x64" => existing_assets
                    .iter()
                    .any(|(name, _)| name.ends_with(".deb") && name.contains("amd64")),
                "release_rpm_arm" => existing_assets
                    .iter()
                    .any(|(name, _)| name.ends_with(".rpm") && name.contains("aarch64")),
                "release_rpm_x64" => existing_assets
                    .iter()
                    .any(|(name, _)| name.ends_with(".rpm") && name.contains("x86_64")),
                "release_docker_arm" | "release_docker_x64" | "release_docker_manifest" => {
                    // Docker jobs don't upload artifacts, always run them
                    false
                }
                _ => false,
            };

            if job_has_assets {
                println!("   ⏭️  Skipping {} (assets already exist)", job.name);
                false
            } else {
                println!("   ✅ Will run: {}", job.name);
                true
            }
        })
        .collect();

    if jobs.is_empty() {
        println!("\n✅ All assets already exist, nothing to build!");
        return Ok(());
    }

    println!("   {} job(s) to execute", jobs.len());

    // Step 1: Trigger the pipeline
    println!("\n📡 Step 1: Triggering CI pipeline...");
    let pipeline = crate::gitlab::trigger_pipeline(
        PROJECT_ID.to_string(),
        branch.clone(),
        vec![("NETWORK".to_string(), network.clone())],
    )
    .await?;

    println!("✅ Pipeline triggered successfully!");
    println!("   Pipeline ID: {}", pipeline.id);
    println!("   Pipeline URL: {}", pipeline.web_url);

    // Step 2: Wait for pipeline to be ready and get job IDs
    println!("\n⏳ Step 2: Waiting for pipeline to initialize...");
    tokio::time::sleep(Duration::from_secs(15)).await;

    let job_ids = get_job_ids(&pipeline.id, &jobs).await?;
    println!("✅ Found {} jobs in pipeline", job_ids.len());

    // Step 3: Wait a bit more for jobs to transition to manual state
    println!("\n⏳ Waiting for jobs to become manual...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Step 4: Play (trigger) each manual job
    // Note: release_docker_manifest will start automatically when its dependencies complete
    println!("\n▶️  Step 3: Triggering release jobs...");
    for (job_name, job_id) in &job_ids {
        // Skip the manifest job as it starts automatically via needs/when: on_success
        if job_name == "release_docker_manifest" {
            println!(
                "   ⏭️  Skipping {} (will start automatically after Docker builds)",
                job_name
            );
            continue;
        }

        println!("   Starting job: {} (ID: {})", job_name, job_id);

        // Try multiple times to play the job
        for attempt in 1..=3 {
            match crate::gitlab::play_job(PROJECT_ID.to_string(), *job_id).await {
                Ok(_) => {
                    println!("   ✅ Job {} started", job_name);
                    break;
                }
                Err(e) => {
                    if attempt < 3 {
                        println!("   ⏳ Attempt {}/3: Job not ready yet, waiting...", attempt);
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    } else {
                        println!(
                            "   ⚠️  Warning: Failed to start job {} after 3 attempts: {}",
                            job_name, e
                        );
                        println!("   You may need to start this job manually on GitLab");
                    }
                }
            }
        }
    }

    // Step 5: Monitor jobs and collect results
    println!("\n👀 Step 4: Monitoring job execution...");
    let job_results = monitor_jobs(&pipeline.id, &jobs).await?;

    // Count job statuses
    let success_count = job_results.iter().filter(|r| r.status == "success").count();
    let failed_count = job_results.iter().filter(|r| r.status == "failed").count();
    let canceled_count = job_results
        .iter()
        .filter(|r| r.status == "canceled")
        .count();

    // Check if we should continue
    if success_count == 0 {
        println!("\n❌ Release build process failed!");
        println!("📋 Summary:");
        println!("   - Network: {}", network);
        println!("   - Branch: {}", branch);
        println!("   - Release tag: {}", release_tag);
        println!("   - Pipeline: {}", pipeline.web_url);
        println!(
            "   - Jobs succeeded: {}/{}",
            success_count,
            job_results.len()
        );
        if failed_count > 0 {
            println!("   - Jobs failed: {}", failed_count);
        }
        if canceled_count > 0 {
            println!("   - Jobs canceled: {}", canceled_count);
        }
        return Err(anyhow!(
            "No jobs completed successfully. Please check the pipeline: {}",
            pipeline.web_url
        ));
    }

    // Step 6: Download artifacts from successful jobs
    println!("\n📥 Step 5: Downloading artifacts from successful jobs...");
    let artifacts_dir = PathBuf::from("target/release-artifacts");
    std::fs::create_dir_all(&artifacts_dir)?;

    let mut all_artifacts = Vec::new();
    for result in &job_results {
        println!(
            "   Job: {} - Status: {} - Has artifact patterns: {}",
            result.job_name,
            result.status,
            !result.artifact_patterns.is_empty()
        );

        if result.status == "success" && !result.artifact_patterns.is_empty() {
            println!("   Downloading artifacts from job: {}", result.job_name);
            let downloaded = download_job_artifacts(result, &artifacts_dir).await?;
            println!(
                "   Downloaded {} files from {}",
                downloaded.len(),
                result.job_name
            );
            all_artifacts.extend(downloaded);
        } else if result.status == "success" && result.artifact_patterns.is_empty() {
            println!(
                "   ⚠️  Job {} succeeded but has no artifact patterns defined",
                result.job_name
            );
        }
    }

    println!("   Total artifacts collected: {}", all_artifacts.len());

    // Step 7: Upload artifacts to GitLab release
    println!("\n📤 Step 6: Uploading artifacts to GitLab release...");
    upload_artifacts_to_release(&release_tag, &all_artifacts).await?;

    // Step 8: Summary
    println!("\n✅ Release build process completed successfully!");
    println!("📋 Summary:");
    println!("   - Network: {}", network);
    println!("   - Branch: {}", branch);
    println!("   - Release tag: {}", release_tag);
    println!("   - Pipeline: {}", pipeline.web_url);

    println!(
        "   - Jobs succeeded: {}/{}",
        success_count,
        job_results.len()
    );
    if failed_count > 0 {
        println!("   - Jobs failed: {}", failed_count);
    }
    if canceled_count > 0 {
        println!("   - Jobs canceled: {}", canceled_count);
    }
    if failed_count > 0 || canceled_count > 0 {
        println!("   ⚠️  Some jobs did not complete, but successful artifacts were uploaded");
    }
    println!("   - Artifacts uploaded: {}", all_artifacts.len());

    Ok(())
}

/// Get job IDs for specified jobs in a pipeline
async fn get_job_ids(pipeline_id: &u64, job_configs: &[JobConfig]) -> Result<HashMap<String, u64>> {
    let mut job_ids = HashMap::new();
    let mut retries = 0;

    while retries < MAX_RETRIES {
        let jobs = crate::gitlab::get_pipeline_jobs(PROJECT_ID.to_string(), *pipeline_id).await?;

        for config in job_configs {
            if let Some(job) = jobs.iter().find(|j| j.name == config.name) {
                job_ids.insert(config.name.clone(), job.id);
            }
        }

        if job_ids.len() == job_configs.len() {
            return Ok(job_ids);
        }

        retries += 1;
        if retries < MAX_RETRIES {
            println!(
                "   Found {}/{} jobs, retrying in {}s...",
                job_ids.len(),
                job_configs.len(),
                POLL_INTERVAL_SECS
            );
            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    Err(anyhow!(
        "Could not find all required jobs after {} retries. Found {}/{} jobs.",
        MAX_RETRIES,
        job_ids.len(),
        job_configs.len()
    ))
}

/// Monitor jobs until they complete (success or failure)
async fn monitor_jobs(pipeline_id: &u64, job_configs: &[JobConfig]) -> Result<Vec<JobResult>> {
    let mut completed_jobs: HashMap<String, JobResult> = HashMap::new();
    let total_jobs = job_configs.len();
    let mut poll_count = 0u32;

    loop {
        let jobs = crate::gitlab::get_pipeline_jobs(PROJECT_ID.to_string(), *pipeline_id).await?;

        for config in job_configs {
            // Skip if already completed
            if completed_jobs.contains_key(&config.name) {
                continue;
            }

            if let Some(job) = jobs.iter().find(|j| j.name == config.name) {
                match job.status.as_str() {
                    "success" => {
                        println!("   ✅ Job completed successfully: {}", config.name);
                        completed_jobs.insert(
                            config.name.clone(),
                            JobResult {
                                job_name: config.name.clone(),
                                job_id: job.id,
                                status: "success".to_string(),
                                artifact_patterns: config.artifact_patterns.clone(),
                            },
                        );
                    }
                    "failed" => {
                        println!("   ❌ Job failed: {}", config.name);
                        completed_jobs.insert(
                            config.name.clone(),
                            JobResult {
                                job_name: config.name.clone(),
                                job_id: job.id,
                                status: "failed".to_string(),
                                artifact_patterns: Vec::new(),
                            },
                        );
                    }
                    "canceled" => {
                        println!("   🚫 Job canceled: {}", config.name);
                        completed_jobs.insert(
                            config.name.clone(),
                            JobResult {
                                job_name: config.name.clone(),
                                job_id: job.id,
                                status: "canceled".to_string(),
                                artifact_patterns: Vec::new(),
                            },
                        );
                    }
                    "skipped" => {
                        println!("   ⏭️  Job skipped: {}", config.name);
                        completed_jobs.insert(
                            config.name.clone(),
                            JobResult {
                                job_name: config.name.clone(),
                                job_id: job.id,
                                status: "skipped".to_string(),
                                artifact_patterns: Vec::new(),
                            },
                        );
                    }
                    status => {
                        // Job is still running (pending, created, running, etc.)
                        if poll_count.is_multiple_of(5) {
                            // Print status every 5 polls
                            println!(
                                "   ⏳ Job in progress: {} (status: {})",
                                config.name, status
                            );
                        }
                    }
                }
            }
        }

        if completed_jobs.len() == total_jobs {
            break;
        }

        poll_count += 1;
        // Wait before next poll
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }

    Ok(completed_jobs.into_values().collect())
}

/// Download artifacts from a successful job
async fn download_job_artifacts(
    job_result: &JobResult,
    artifacts_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let zip_path = artifacts_dir.join(format!("{}.zip", job_result.job_name));
    let extract_dir = artifacts_dir.join(&job_result.job_name);

    // Download artifacts zip
    match crate::gitlab::download_job_artifacts(
        PROJECT_ID.to_string(),
        job_result.job_id,
        &zip_path,
    )
    .await
    {
        Ok(_) => {
            println!("      Downloaded: {}", zip_path.display());

            // Extract artifacts
            match crate::gitlab::extract_zip(&zip_path, &extract_dir) {
                Ok(_) => {
                    println!("      Extracted to: {}", extract_dir.display());

                    // Find all files in extract directory
                    let mut artifacts = Vec::new();
                    find_artifact_files(&extract_dir, &mut artifacts)?;

                    println!("      Found {} artifact file(s)", artifacts.len());
                    Ok(artifacts)
                }
                Err(e) => {
                    println!("      ⚠️  Warning: Failed to extract artifacts: {}", e);
                    Ok(Vec::new())
                }
            }
        }
        Err(e) => {
            println!("      ⚠️  Warning: Failed to download artifacts: {}", e);
            Ok(Vec::new())
        }
    }
}

/// Recursively find all files in a directory
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

/// Upload artifacts to GitLab release
async fn upload_artifacts_to_release(release_tag: &str, artifacts: &[PathBuf]) -> Result<()> {
    for artifact_path in artifacts {
        let file_name = artifact_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid artifact path: {:?}", artifact_path))?
            .to_string_lossy()
            .to_string();

        // Skip non-package files
        if !file_name.ends_with(".deb") && !file_name.ends_with(".rpm") {
            continue;
        }

        println!("   Uploading: {}", file_name);

        // Upload file to GitLab
        match crate::gitlab::upload_file(PROJECT_ID.to_string(), artifact_path, file_name.clone())
            .await
        {
            Ok(asset_url) => {
                println!("      Uploaded: {}", asset_url);

                // Create asset link in release
                match crate::gitlab::create_asset_link(
                    release_tag.to_string(),
                    file_name.clone(),
                    asset_url,
                )
                .await
                {
                    Ok(_) => println!("      ✅ Asset link created"),
                    Err(e) => println!("      ⚠️  Warning: Failed to create asset link: {}", e),
                }
            }
            Err(e) => println!("      ⚠️  Warning: Failed to upload file: {}", e),
        }
    }

    Ok(())
}

/// Compute release tag from network name and client version
/// Format: {runtime}-{runtime_version}-{client_version}
/// Example: gtest-1100-0.12.0
fn compute_release_tag(network: &str) -> Result<String> {
    // Get client version from node/Cargo.toml
    let client_version = get_client_version()?;

    // Release tag = network name + client version
    // Example: gtest-1100 + 0.12.0 = gtest-1100-0.12.0
    Ok(format!("{}-{}", network, client_version))
}

fn get_client_version() -> Result<String> {
    use std::process::Command;

    let output = Command::new("grep")
        .args(["version", "node/Cargo.toml"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to read client version from node/Cargo.toml"
        ));
    }

    let version_line = String::from_utf8(output.stdout)?;
    let version = version_line
        .split("version = \"")
        .nth(1)
        .ok_or_else(|| anyhow!("Invalid version format in node/Cargo.toml"))?
        .split('"')
        .next()
        .ok_or_else(|| anyhow!("Invalid version format in node/Cargo.toml"))?;

    Ok(version.to_string())
}
