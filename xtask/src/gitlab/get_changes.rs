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
    query_path = "res/get_changes.gql",
    response_derives = "Debug"
)]
pub struct GetChangesQuery;

pub(super) async fn get_changes(milestone: String) -> Result<String> {
    // this is the important line
    let request_body = GetChangesQuery::build_query(get_changes_query::Variables { milestone });

    let client = reqwest::Client::new();
    let res = client
        .post("https://git.duniter.org/api/graphql")
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<get_changes_query::ResponseData> = res.json().await?;

    if let Some(data) = response_body.data {
        if let Some(project) = data.project {
            if let Some(merge_requests) = project.merge_requests {
                if let Some(nodes) = merge_requests.nodes {
                    let mut changes = String::new();
                    for merge_request in nodes.into_iter().flatten() {
                        changes.push_str(&format!(
                            "* {mr_title} (!{mr_number})\n",
                            mr_title = merge_request.title,
                            mr_number = merge_request.iid
                        ));
                    }
                    Ok(changes)
                } else {
                    Err(anyhow!("No changes found"))
                }
            } else {
                Err(anyhow!("No changes found"))
            }
        } else {
            Err(anyhow!("Project not found"))
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
