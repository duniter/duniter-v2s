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
    query_path = "res/get_release.gql",
    response_derives = "Debug"
)]
pub struct GetReleaseOfProjectQuery;

pub(super) async fn get_release(tag: String) -> Result<Vec<String>> {
    // this is the important line
    let request_body =
        GetReleaseOfProjectQuery::build_query(get_release_of_project_query::Variables { tag });

    let client = reqwest::Client::new();
    let res = client
        .post("https://git.duniter.org/api/graphql")
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<get_release_of_project_query::ResponseData> = res.json().await?;

    if let Some(data) = response_body.data {
        Ok(data
            .project
            .expect("should have project")
            .release
            .expect("should have release")
            .assets
            .expect("should have assets")
            .links
            .expect("should have links")
            .edges
            .expect("should have edges")
            .into_iter()
            .map(|edge| {
                edge.expect("should have edge")
                    .node
                    .expect("should have node")
                    .direct_asset_url
                    .expect("should have directAssetUrl")
            })
            .collect())
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
