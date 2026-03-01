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
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PipelineStatus {
    pub id: u64,
    pub iid: u64,
    pub status: String,
    pub web_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JobStatus {
    pub id: u64,
    pub name: String,
    pub status: String,
    #[allow(dead_code)]
    pub stage: String,
    #[allow(dead_code)]
    pub web_url: String,
}

/// Get the status of a pipeline
#[allow(dead_code)]
pub async fn get_pipeline_status(
    gitlab_token: String,
    project_id: String,
    pipeline_id: u64,
) -> Result<PipelineStatus> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "https://git.duniter.org/api/v4/projects/{project_id}/pipelines/{pipeline_id}"
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .send()
        .await?;

    if response.status().is_success() {
        let pipeline: PipelineStatus = response.json().await?;
        Ok(pipeline)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(anyhow!(
            "Failed to get pipeline status (status {}): {}",
            status,
            error_text
        ))
    }
}

/// Get all jobs for a pipeline
pub async fn get_pipeline_jobs(
    gitlab_token: String,
    project_id: String,
    pipeline_id: u64,
) -> Result<Vec<JobStatus>> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "https://git.duniter.org/api/v4/projects/{project_id}/pipelines/{pipeline_id}/jobs"
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .send()
        .await?;

    if response.status().is_success() {
        let jobs: Vec<JobStatus> = response.json().await?;
        Ok(jobs)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(anyhow!(
            "Failed to get pipeline jobs (status {}): {}",
            status,
            error_text
        ))
    }
}

/// Play (trigger) a manual job
pub async fn play_job(gitlab_token: String, project_id: String, job_id: u64) -> Result<JobStatus> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!(
            "https://git.duniter.org/api/v4/projects/{project_id}/jobs/{job_id}/play"
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .send()
        .await?;

    if response.status().is_success() {
        let job: JobStatus = response.json().await?;
        Ok(job)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(anyhow!(
            "Failed to play job (status {}): {}",
            status,
            error_text
        ))
    }
}
