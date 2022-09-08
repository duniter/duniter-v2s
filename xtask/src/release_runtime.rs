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

mod create_release;
mod get_changes;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::io::Read;
use std::process::Command;

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
    // Get current dir
    let pwd = std::env::current_dir()?
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("Fail to read current dir path: invalid utf8 string!"))?;

    // TODO: check spec_version in the code and bump if necessary (with a commit)

    // TODO: create and push a git tag runtime-{spec_version}

    // Create target folder for runtime build
    Command::new("mkdir")
        .args(["-p", "runtime/gdev/target"])
        .status()?;
    Command::new("chmod")
        .args(["777", "runtime/gdev/target"])
        .status()?;

    // Build the new runtime
    println!("Build gdev-runtime… (take a while)");
    let output = Command::new("docker")
        .args([
            "run",
            "-i",
            "--rm",
            "-e",
            "PACKAGE=gdev-runtime",
            "-e",
            "RUNTIME_DIR=runtime/gdev",
            "-v",
            &format!("{}:/build", pwd),
            "paritytech/srtool:1.62.0",
            "build",
            "--app",
            "--json",
            "-cM",
        ])
        .output()?;

    // Read the srtool json output
    let srtool: Srtool = serde_json::from_str(
        std::str::from_utf8(&output.stdout)?
            .lines()
            .last()
            .ok_or_else(|| anyhow!("empty srtool output"))?,
    )
    .with_context(|| "Fail to parse srtool json output")?;

    // Generate release notes
    let release_notes = gen_release_notes(spec_version, srtool)
        .await
        .with_context(|| "Fail to generate release notes")?;

    // TODO: Call gitlab API to publish the release notes (and upload the wasm)
    println!("{}", release_notes);
    let gitlab_token =
        std::env::var("GITLAB_TOKEN").with_context(|| "missing env var GITLAB_TOKEN")?;
    create_release::create_release(gitlab_token, spec_version, release_notes).await?;

    Ok(())
}

async fn gen_release_notes(spec_version: u32, srtool: Srtool) -> Result<String> {
    // Read template file
    const RELEASE_NOTES_TEMPLATE_FILEPATH: &str = "xtask/res/runtime_release_notes.template";
    let mut file = std::fs::File::open(RELEASE_NOTES_TEMPLATE_FILEPATH)?;
    let mut template = String::new();
    file.read_to_string(&mut template)?;

    // Prepare srtool values
    let uncompressed_size = srtool.runtimes.compact.subwasm.size;
    let wasm = srtool.runtimes.compressed.subwasm;
    let compression_percent = (1.0 - (wasm.size as f64 / uncompressed_size as f64)) * 100.0;

    // Get changes (list of MRs) from gitlab API
    let changes = get_changes::get_changes(spec_version).await?;

    // Fill template values
    let mut values = std::collections::HashMap::new();
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
    values.insert("changes".to_owned(), changes);

    // Render template
    placeholder::render(&template, &values).map_err(|e| anyhow!("Fail to render template: {}", e))
}
