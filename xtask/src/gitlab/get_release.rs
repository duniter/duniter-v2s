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

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    assets: Assets,
}

#[derive(Debug, Deserialize)]
struct Assets {
    links: Vec<AssetLink>,
}

#[derive(Debug, Deserialize)]
struct AssetLink {
    name: String,
    url: String,
}

/// Get release assets (name and URL) using GitLab REST API
pub(super) async fn get_release(tag: String) -> Result<Vec<(String, String)>> {
    let gitlab_token = std::env::var("GITLAB_TOKEN")
        .map_err(|_| anyhow!("GITLAB_TOKEN environment variable not set"))?;

    let client = reqwest::Client::new();
    let project_id = "nodes%2Frust%2Fduniter-v2s";

    let res = client
        .get(format!(
            "https://git.duniter.org/api/v4/projects/{project_id}/releases/{tag}"
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .send()
        .await?;

    if res.status().is_success() {
        let release: ReleaseResponse = res.json().await?;
        Ok(release
            .assets
            .links
            .into_iter()
            .map(|link| (link.name, link.url))
            .collect())
    } else {
        let status = res.status();
        let error_text = res.text().await?;
        Err(anyhow!(
            "Failed to fetch release (status {}): {}",
            status,
            error_text
        ))
    }
}
