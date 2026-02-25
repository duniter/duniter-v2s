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
use std::time::Duration;

const SQUID_PROJECT_ID: &str = "nodes%2Fduniter-squid";
const POLL_INTERVAL_SECS: u64 = 30;
const MAX_INIT_RETRIES: u32 = 5;

/// Expected jobs in the squid CI pipeline
const PREPARE_JOB: &str = "prepare";
const BUILD_JOBS: &[&str] = &["build:squid", "build:graphile", "build:postgres"];

/// Triggers the duniter-squid CI pipeline to build and push Docker images to Docker Hub.
///
/// The squid CI pipeline has two stages:
/// 1. `prepare` — downloads genesis data from the duniter-v2s release, generates genesis.json,
///    and fetches substrate metadata from a live RPC node.
/// 2. `build` — builds and pushes three multi-arch Docker images:
///    squid-app, squid-graphile, squid-postgres.
///
/// # Arguments
/// * `release_tag` - The duniter-v2s release tag (e.g., "g1-1000")
/// * `branch` - The squid Git branch to build from (e.g., "main")
/// * `rpc_url` - Optional RPC endpoint override for metadata fetching
pub async fn trigger_squid_builds(
    release_tag: String,
    branch: String,
    rpc_url: Option<String>,
) -> Result<()> {
    // Validate release_tag format (e.g., g1-1000, gtest-1000, gdev-800)
    let network = release_tag
        .split('-')
        .next()
        .ok_or_else(|| anyhow!("Invalid release tag format: {}", release_tag))?;

    match network {
        "g1" | "gtest" | "gdev" => {}
        _ => {
            return Err(anyhow!(
                "Unknown network '{}' in release tag '{}'. Expected g1, gtest, or gdev.",
                network,
                release_tag
            ));
        }
    }

    println!(
        "🦑 Starting squid build process for release: {}",
        release_tag
    );
    println!("   Network: {}", network);
    println!("   Branch: {}", branch);
    if let Some(ref url) = rpc_url {
        println!("   RPC override: {}", url);
    }

    // Step 1: Trigger the pipeline
    println!("\n📡 Step 1: Triggering squid CI pipeline...");
    let mut variables = vec![("RELEASE_TAG".to_string(), release_tag.clone())];
    if let Some(url) = rpc_url {
        variables.push(("RPC_URL".to_string(), url));
    }
    let pipeline =
        crate::gitlab::trigger_pipeline(SQUID_PROJECT_ID.to_string(), branch.clone(), variables)
            .await?;

    println!("   Pipeline ID: {}", pipeline.id);
    println!("   Pipeline URL: {}", pipeline.web_url);

    // Step 2: Wait for jobs to appear
    println!("\n⏳ Step 2: Waiting for pipeline jobs to initialize...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    let all_expected_jobs: Vec<&str> = std::iter::once(PREPARE_JOB)
        .chain(BUILD_JOBS.iter().copied())
        .collect();

    let job_ids = wait_for_jobs(&pipeline.id, &all_expected_jobs).await?;
    println!("   Found {} jobs in pipeline", job_ids.len());

    // Step 3: Play the prepare job (it's manual when triggered via API)
    println!("\n▶️  Step 3: Starting prepare job...");
    let prepare_job_id = *job_ids
        .get(PREPARE_JOB)
        .ok_or_else(|| anyhow!("Could not find '{}' job", PREPARE_JOB))?;

    play_job_with_retries(PREPARE_JOB, prepare_job_id).await?;

    // Step 4: Wait for prepare to complete
    println!("\n👀 Step 4: Waiting for prepare job to complete...");
    let prepare_status = wait_for_job_completion(&pipeline.id, PREPARE_JOB).await?;

    if prepare_status != "success" {
        return Err(anyhow!(
            "Prepare job failed with status: {}.\n\
             Check the pipeline: {}",
            prepare_status,
            pipeline.web_url
        ));
    }
    println!("   Prepare job completed successfully");

    // Step 5: Play build jobs
    println!("\n▶️  Step 5: Starting build jobs...");
    for job_name in BUILD_JOBS {
        if let Some(&job_id) = job_ids.get(*job_name) {
            play_job_with_retries(job_name, job_id).await?;
        }
    }

    // Step 6: Monitor build jobs
    println!("\n👀 Step 6: Monitoring build jobs...");
    let mut success_count = 0;
    let mut failed_jobs = Vec::new();

    loop {
        let jobs =
            crate::gitlab::get_pipeline_jobs(SQUID_PROJECT_ID.to_string(), pipeline.id).await?;

        let mut all_done = true;
        for job_name in BUILD_JOBS {
            if let Some(job) = jobs.iter().find(|j| j.name == *job_name) {
                match job.status.as_str() {
                    "success" | "failed" | "canceled" | "skipped" => {}
                    _ => {
                        all_done = false;
                    }
                }
            }
        }

        if all_done {
            // Collect final results
            for job_name in BUILD_JOBS {
                if let Some(job) = jobs.iter().find(|j| j.name == *job_name) {
                    match job.status.as_str() {
                        "success" => {
                            println!("   ✅ {}", job_name);
                            success_count += 1;
                        }
                        status => {
                            println!("   ❌ {} ({})", job_name, status);
                            failed_jobs.push(job_name.to_string());
                        }
                    }
                }
            }
            break;
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }

    // Summary
    println!("\n📋 Summary:");
    println!("   - Release tag: {}", release_tag);
    println!("   - Network: {}", network);
    println!("   - Branch: {}", branch);
    println!("   - Pipeline: {}", pipeline.web_url);
    println!(
        "   - Build jobs: {}/{} succeeded",
        success_count,
        BUILD_JOBS.len()
    );

    if !failed_jobs.is_empty() {
        println!("   - Failed: {}", failed_jobs.join(", "));
        return Err(anyhow!(
            "Some build jobs failed. Check the pipeline: {}",
            pipeline.web_url
        ));
    }

    println!("\n✅ Squid Docker images built and pushed to Docker Hub!");
    println!("   - duniter/squid-app-{}", network);
    println!("   - duniter/squid-graphile-{}", network);
    println!("   - duniter/squid-postgres-{}", network);

    Ok(())
}

/// Wait for all expected jobs to appear in the pipeline
async fn wait_for_jobs(
    pipeline_id: &u64,
    expected_jobs: &[&str],
) -> Result<std::collections::HashMap<String, u64>> {
    let mut job_ids = std::collections::HashMap::new();

    for attempt in 1..=MAX_INIT_RETRIES {
        let jobs =
            crate::gitlab::get_pipeline_jobs(SQUID_PROJECT_ID.to_string(), *pipeline_id).await?;

        for job_name in expected_jobs {
            if let Some(job) = jobs.iter().find(|j| j.name == *job_name) {
                job_ids.insert(job_name.to_string(), job.id);
            }
        }

        if job_ids.len() == expected_jobs.len() {
            return Ok(job_ids);
        }

        if attempt < MAX_INIT_RETRIES {
            println!(
                "   Found {}/{} jobs, retrying in {}s... (attempt {}/{})",
                job_ids.len(),
                expected_jobs.len(),
                POLL_INTERVAL_SECS,
                attempt,
                MAX_INIT_RETRIES
            );
            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    Err(anyhow!(
        "Could not find all expected jobs after {} retries. Found {}/{}: {:?}",
        MAX_INIT_RETRIES,
        job_ids.len(),
        expected_jobs.len(),
        job_ids.keys().collect::<Vec<_>>()
    ))
}

/// Play a manual job with retries
async fn play_job_with_retries(job_name: &str, job_id: u64) -> Result<()> {
    for attempt in 1..=3 {
        match crate::gitlab::play_job(SQUID_PROJECT_ID.to_string(), job_id).await {
            Ok(_) => {
                println!("   ✅ Started: {}", job_name);
                return Ok(());
            }
            Err(e) => {
                if attempt < 3 {
                    println!(
                        "   ⏳ {} not ready yet (attempt {}/3), waiting...",
                        job_name, attempt
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                } else {
                    println!(
                        "   ⚠️  Warning: Failed to start {} after 3 attempts: {}",
                        job_name, e
                    );
                    println!("   You may need to start this job manually on GitLab");
                }
            }
        }
    }
    Ok(())
}

/// Wait for a specific job to complete
async fn wait_for_job_completion(pipeline_id: &u64, job_name: &str) -> Result<String> {
    loop {
        let jobs =
            crate::gitlab::get_pipeline_jobs(SQUID_PROJECT_ID.to_string(), *pipeline_id).await?;

        if let Some(job) = jobs.iter().find(|j| j.name == job_name) {
            match job.status.as_str() {
                "success" | "failed" | "canceled" | "skipped" => {
                    return Ok(job.status.clone());
                }
                _ => {}
            }
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}
