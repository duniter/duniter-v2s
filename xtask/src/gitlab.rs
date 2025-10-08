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

mod create_asset_link;
mod create_network_release;
mod create_release;
mod get_changes;
mod get_issues;
mod get_release;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::fs;

#[derive(Default, Deserialize)]
struct Srtool {
    r#gen: String,
    rustc: String,
    runtimes: SrtoolRuntimes,
}

#[derive(Default, Deserialize)]
struct SrtoolRuntimes {
    compact: SrtoolRuntime,
    compressed: SrtoolRuntime,
}

#[derive(Default, Deserialize)]
struct SrtoolRuntime {
    subwasm: SrtoolRuntimeSubWasm,
}

#[derive(Default, Deserialize)]
struct SrtoolRuntimeSubWasm {
    core_version: CoreVersion,
    metadata_version: u32,
    size: u32,
    blake2_256: String,
    proposal_hash: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoreVersion {
    //impl_name: String,
    //impl_version: u32,
    spec_name: String,
    spec_version: u32,
    //transaction_version: u32,
}

pub(super) async fn release_network(network: String, branch: String) -> Result<()> {
    let mut release_notes = String::from(
        "
# Runtime

",
    );
    add_srtool_notes(network.clone(), &mut release_notes)?;

    println!("{}", release_notes);
    let gitlab_token =
        std::env::var("GITLAB_TOKEN").with_context(|| "missing env var GITLAB_TOKEN")?;
    create_network_release::create_network_release(
        gitlab_token,
        branch,
        network,
        release_notes.to_string(),
    )
    .await?;

    Ok(())
}

pub(super) async fn release_runtime(
    name: String,
    network: String,
    branch: String,
    milestone: String,
) -> Result<()> {
    release(
        "Runtime".to_string(),
        name,
        Some(network),
        branch,
        milestone,
    )
    .await
}

pub(super) async fn release_client(name: String, branch: String, milestone: String) -> Result<()> {
    release("Client".to_string(), name, None, branch, milestone).await
}

async fn release(
    title: String,
    name: String,
    network: Option<String>,
    branch: String,
    milestone: String,
) -> Result<()> {
    let mut release_notes = format!(
        "
# {title}

"
    );

    if let Some(network) = network {
        add_srtool_notes(network.clone(), &mut release_notes)?;
    }

    // Get changes (list of MRs) from gitlab API
    let changes = get_changes::get_changes(milestone.clone()).await?;

    release_notes.push_str(
        format!(
            "
# Changes

{changes}
"
        )
        .as_str(),
    );

    // Get changes (list of MRs) from gitlab API
    let issues = get_issues::get_issues(milestone.clone()).await?;

    release_notes.push_str(
        format!(
            "
# Issues

{issues}
"
        )
        .as_str(),
    );
    println!("{}", release_notes);
    let gitlab_token =
        std::env::var("GITLAB_TOKEN").with_context(|| "missing env var GITLAB_TOKEN")?;
    create_release::create_release(
        gitlab_token,
        name,
        branch,
        milestone,
        release_notes.to_string(),
    )
    .await?;

    Ok(())
}

fn add_srtool_notes(network: String, release_notes: &mut String) -> Result<()> {
    // Generate release notes
    let currency = network.clone();
    let env_var = "SRTOOL_OUTPUT".to_string();
    let sr_tool_output_file =
        std::env::var(env_var).unwrap_or_else(|_| "release/network_srtool_output.json".to_string());

    let read = fs::read_to_string(sr_tool_output_file);
    match read {
        Ok(sr_tool_output) => {
            release_notes.push_str(
                gen_release_notes(currency.to_string(), sr_tool_output)
                    .with_context(|| format!("Fail to generate release notes for {}", currency))?
                    .as_str(),
            );
        }
        Err(e) => {
            eprintln!("srtool JSON output could not be read ({}). Skipped.", e)
        }
    }
    Ok(())
}

pub(super) async fn print_spec(network: String) -> Result<()> {
    let spec_file = match network.clone() {
        network if network.starts_with("g1") => "g1.json",
        network if network.starts_with("gtest") => "gtest.json",
        network if network.starts_with("gdev") => "gdev.json",
        _ => {
            return Err(anyhow!("Invalid network"));
        }
    };
    let assets = get_release::get_release(network).await?;
    if let Some(gdev_spec) = assets.iter().find(|asset| asset.ends_with(spec_file)) {
        let client = reqwest::Client::new();
        let res = client.get(gdev_spec).send().await?;
        let spec = String::from_utf8(res.bytes().await?.to_vec())?;
        println!("{}", spec);
    }
    Ok(())
}

fn gen_release_notes(currency: String, srtool_output: String) -> Result<String> {
    println!("Read srtool output… ");

    // Read the srtool json output
    let srtool_str = srtool_output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .last()
        .ok_or_else(|| anyhow!("empty srtool output"));
    let srtool: Srtool =
        serde_json::from_str(srtool_str?).with_context(|| "Fail to parse srtool json output")?;

    // Read template file
    let template = String::from(
        "
## {currency}

```
🔨 Srtool version: {srtool_version}
🦀 Rustc version: {rustc_version}
🏋️ Runtime Size: {runtime_human_size} ({runtime_size} bytes)
🔥 Core Version: {core_version}
🗜 Compressed: Yes: {compression_percent} %
🎁 Metadata version: {metadata_version}
🗳️ system.setCode hash: {proposal_hash}
#️⃣ Blake2-256 hash:  {blake2_256}
```
",
    );

    // Prepare srtool values
    let uncompressed_size = srtool.runtimes.compact.subwasm.size;
    let wasm = srtool.runtimes.compressed.subwasm;
    let compression_percent = (1.0 - (wasm.size as f64 / uncompressed_size as f64)) * 100.0;

    // Fill template values
    let mut values = std::collections::HashMap::new();
    values.insert("currency".to_owned(), currency);
    values.insert("srtool_version".to_owned(), srtool.r#gen);
    values.insert("rustc_version".to_owned(), srtool.rustc);
    values.insert(
        "runtime_human_size".to_owned(),
        format!("{} KB", wasm.size / 1_024),
    );
    values.insert("runtime_size".to_owned(), wasm.size.to_string());
    values.insert(
        "core_version".to_owned(),
        format!(
            "{}-{}",
            wasm.core_version.spec_name, wasm.core_version.spec_version
        ),
    );
    values.insert(
        "compression_percent".to_owned(),
        format!("{:.2}", compression_percent),
    );
    values.insert(
        "metadata_version".to_owned(),
        wasm.metadata_version.to_string(),
    );
    values.insert("proposal_hash".to_owned(), wasm.proposal_hash);
    values.insert("blake2_256".to_owned(), wasm.blake2_256);

    // Render template
    placeholder::render(&template, &values).map_err(|e| anyhow!("Fail to render template: {}", e))
}

pub(crate) async fn create_asset_link(
    tag: String,
    asset_name: String,
    asset_url: String,
) -> Result<()> {
    let gitlab_token =
        std::env::var("GITLAB_TOKEN").with_context(|| "missing env var GITLAB_TOKEN")?;
    create_asset_link::create_asset_link(gitlab_token, tag, asset_name, asset_url).await?;

    Ok(())
}
