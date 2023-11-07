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
mod create_release;
mod get_changes;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Default, Deserialize)]
struct Srtool {
    gen: String,
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

pub(super) async fn release_runtime(spec_version: u32) -> Result<()> {
    // TODO: check spec_version in the code and bump if necessary (with a commit)
    // TODO: create and push a git tag runtime-{spec_version}

    let mut release_notes = String::from("# Runtimes

The runtimes have been built using [{srtool_version}](https://github.com/paritytech/srtool) and `{rustc_version}`.

");

    // Generate release notes
    let runtimes = vec![
        ("ĞDev", "SRTOOL_OUTPUT_GDEV"),
        ("ĞTest", "SRTOOL_OUTPUT_GTEST"),
        ("Ğ1", "SRTOOL_OUTPUT_G1"),
    ];
    for (currency, env_var) in runtimes {
        if let Ok(sr_tool_output_file) = std::env::var(env_var) {
            let read = fs::read_to_string(sr_tool_output_file);
            match read {
                Ok(sr_tool_output) => {
                    release_notes.push_str(
                        gen_release_notes(currency.to_string(), sr_tool_output)
                            .with_context(|| {
                                format!("Fail to generate release notes for {}", currency)
                            })?
                            .as_str(),
                    );
                }
                Err(e) => {
                    eprintln!("srtool JSON output could not be read ({}). Skipped.", e)
                }
            }
        }
    }

    // Get changes (list of MRs) from gitlab API
    let changes = get_changes::get_changes(spec_version).await?;

    release_notes.push_str(
        format!(
            "

# Changes

{changes}
"
        )
        .as_str(),
    );
    println!("{}", release_notes);
    let gitlab_token =
        std::env::var("GITLAB_TOKEN").with_context(|| "missing env var GITLAB_TOKEN")?;
    create_release::create_release(gitlab_token, spec_version, release_notes.to_string()).await?;

    Ok(())
}

fn gen_release_notes(currency: String, srtool_output: String) -> Result<String> {
    println!("Read srtool output… ");

    // Read the srtool json output
    let srtool: Srtool = serde_json::from_str(
        srtool_output
            .lines()
            .last()
            .ok_or_else(|| anyhow!("empty srtool output"))?,
    )
    .with_context(|| "Fail to parse srtool json output")?;

    // Read template file
    let template = String::from(
        "
## {currency}

```
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
    values.insert("srtool_version".to_owned(), srtool.gen);
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
    spec_version: u32,
    asset_name: String,
    asset_url: String,
) -> Result<()> {
    let gitlab_token =
        std::env::var("GITLAB_TOKEN").with_context(|| "missing env var GITLAB_TOKEN")?;
    create_asset_link::create_asset_link(gitlab_token, spec_version, asset_name, asset_url).await?;

    Ok(())
}
