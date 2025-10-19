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
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "res/schema.gql",
    query_path = "res/create_asset_link.gql",
    response_derives = "Debug"
)]
pub struct CreateReleaseAssetLinkMutation;

pub(super) async fn create_asset_link(
    gitlab_token: String,
    tag: String,
    asset_name: String,
    asset_url: String,
) -> Result<()> {
    // this is the important line
    let request_body = CreateReleaseAssetLinkMutation::build_query(
        create_release_asset_link_mutation::Variables {
            name: asset_name,
            url: asset_url,
            tag,
        },
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://git.duniter.org/api/graphql")
        .header("Authorization", format!("Bearer {}", gitlab_token))
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<create_release_asset_link_mutation::ResponseData> =
        res.json().await?;

    if let Some(data) = response_body.data {
        if let Some(body) = data.release_asset_link_create {
            if body.errors.is_empty() {
                Ok(())
            } else {
                println!("{} errors:", body.errors.len());
                for error in body.errors {
                    println!("{}", error);
                }
                Err(anyhow!("Logic errors"))
            }
        } else if let Some(errors) = response_body.errors {
            Err(anyhow!("Errors: {:?}", errors))
        } else {
            Err(anyhow!("Invalid response: no release_asset_link_create"))
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
