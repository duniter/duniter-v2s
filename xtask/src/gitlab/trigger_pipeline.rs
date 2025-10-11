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
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct TriggerPipelineRequest {
    r#ref: String,
    variables: Vec<PipelineVariable>,
}

#[derive(Debug, Serialize)]
struct PipelineVariable {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct PipelineResponse {
    pub id: u64,
    #[allow(dead_code)]
    pub iid: u64,
    #[allow(dead_code)]
    pub status: String,
    pub web_url: String,
}

/// Trigger a CI pipeline on GitLab with specific variables
/// # Arguments
/// * `gitlab_token` - The GitLab authentication token
/// * `project_id` - The GitLab project ID (URL-encoded)
/// * `branch` - The branch to run the pipeline on
/// * `variables` - Variables to pass to the pipeline (e.g., NETWORK=gtest-1100)
pub async fn trigger_pipeline(
    gitlab_token: String,
    project_id: String,
    branch: String,
    variables: Vec<(String, String)>,
) -> Result<PipelineResponse> {
    let client = reqwest::Client::new();

    let pipeline_vars: Vec<PipelineVariable> = variables
        .into_iter()
        .map(|(key, value)| PipelineVariable { key, value })
        .collect();

    let request = TriggerPipelineRequest {
        r#ref: branch,
        variables: pipeline_vars,
    };

    let response = client
        .post(format!(
            "https://git.duniter.org/api/v4/projects/{}/pipeline",
            project_id
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let pipeline: PipelineResponse = response.json().await?;
        Ok(pipeline)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(anyhow!(
            "Failed to trigger pipeline (status {}): {}",
            status,
            error_text
        ))
    }
}

