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

use anyhow::{anyhow, Result};
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "res/schema.gql",
    query_path = "res/create_release.gql",
    response_derives = "Debug"
)]
pub struct CreateReleaseMutation;

pub(super) async fn create_release(
    gitlab_token: String,
    spec_version: u32,
    release_notes: String,
    g1_data_url: String,
    srtool_output_url: String,
    gdev_wasm_url: String,
    gdev_raw_spec_url: String,
) -> Result<()> {
    // this is the important line
    let request_body = CreateReleaseMutation::build_query(create_release_mutation::Variables {
        branch: format!("release/runtime-{}", spec_version - (spec_version % 100)),
        description: release_notes,
        milestone: format!("runtime-{}", spec_version),
        links: vec![
            create_release_mutation::ReleaseAssetLinkInput {
                directAssetPath: None,
                linkType: None,
                name: "g1-data.json".to_string(),
                url: g1_data_url,
            },
            create_release_mutation::ReleaseAssetLinkInput {
                directAssetPath: None,
                linkType: None,
                name: "srtool_output.json".to_string(),
                url: srtool_output_url,
            },
            create_release_mutation::ReleaseAssetLinkInput {
                directAssetPath: None,
                linkType: None,
                name: "gdev_runtime.compact.compressed.wasm".to_string(),
                url: gdev_wasm_url,
            },
            create_release_mutation::ReleaseAssetLinkInput {
                directAssetPath: None,
                linkType: None,
                name: "gdev-raw.json".to_string(),
                url: gdev_raw_spec_url,
            },
        ],
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://git.duniter.org/api/graphql")
        .header("PRIVATE-TOKEN", gitlab_token)
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<create_release_mutation::ResponseData> = res.json().await?;

    if let Some(data) = response_body.data {
        if let Some(release_create) = data.release_create {
            if release_create.errors.is_empty() {
                Ok(())
            } else {
                println!("{} errors:", release_create.errors.len());
                for error in release_create.errors {
                    println!("{}", error);
                }
                Err(anyhow!("Logic errors"))
            }
        } else {
            Err(anyhow!("Invalid response: no release_create"))
        }
    } else if let Some(errors) = response_body.errors {
        println!("{} errors:", errors.len());
        for error in errors {
            println!("{}", error);
        }
        Err(anyhow!("GraphQL errors"))
    } else {
        Err(anyhow!("Invalid response: no data nor errors"))
    }
}
