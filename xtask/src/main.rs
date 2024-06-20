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

#![feature(let_chains)]

mod gen_doc;
mod release_runtime;

use anyhow::{Context, Result};
use clap::Parser;
use std::{
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    process::Command,
};

const MIN_RUST_VERSION: &str = "1.58.0";

#[derive(Debug, clap::Parser)]
struct DuniterXTask {
    #[clap(subcommand)]
    command: DuniterXTaskCommand,
}

#[derive(Debug, clap::Subcommand)]
enum DuniterXTaskCommand {
    /// Build duniter binary
    Build {
        #[clap(long)]
        production: bool,
    },
    /// Generate documentation (calls and events)
    GenDoc,
    /// Inject runtime code in raw specs
    InjectRuntimeCode {
        #[clap(short, long)]
        /// Runtime filepath
        runtime: PathBuf,
        #[clap(short = 's', long)]
        /// Raw spec filepath
        raw_spec: PathBuf,
    },
    /// Release a new network
    ReleaseNetwork { network: String, branch: String },
    /// Release a new runtime
    ReleaseRuntime {
        network: String,
        branch: String,
        milestone: String,
    },
    /// Print the chainSpec published on given Network Release
    PrintSpec { network: String },
    /// Create asset in a release
    CreateAssetLink {
        tag: String,
        asset_name: String,
        asset_url: String,
    },
    /// Execute unit tests and integration tests
    /// End2tests are skipped
    Test,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = DuniterXTask::parse();

    if !version_check::is_min_version(MIN_RUST_VERSION).unwrap_or(false)
        && exec_should_success(Command::new("rustup").args(["update", "stable"])).is_err()
    {
        eprintln!(
                "Duniter requires stable Rust {} or higher. If you installed the Rust toolchain via rustup, please execute the command `rustup update stable`.",
                MIN_RUST_VERSION
            );
        std::process::exit(1);
    }

    match &args.command {
        DuniterXTaskCommand::PrintSpec { .. } => { /* no print */ }
        _ => {
            Command::new("rustc").arg("--version").status()?;
            Command::new("cargo").arg("--version").status()?;
        }
    }

    match args.command {
        DuniterXTaskCommand::Build { production } => build(production),
        DuniterXTaskCommand::GenDoc => gen_doc::gen_doc(),
        DuniterXTaskCommand::InjectRuntimeCode { runtime, raw_spec } => {
            inject_runtime_code(&raw_spec, &runtime)
        }
        DuniterXTaskCommand::ReleaseNetwork { network, branch } => {
            release_runtime::release_network(network, branch).await
        }
        DuniterXTaskCommand::ReleaseRuntime {
            network,
            branch,
            milestone,
        } => release_runtime::release_runtime(network, branch, milestone).await,
        DuniterXTaskCommand::PrintSpec { network } => release_runtime::print_spec(network).await,
        DuniterXTaskCommand::CreateAssetLink {
            tag,
            asset_name,
            asset_url,
        } => release_runtime::create_asset_link(tag, asset_name, asset_url).await,
        DuniterXTaskCommand::Test => test(),
    }
}

fn inject_runtime_code(raw_spec: &Path, runtime: &Path) -> Result<()> {
    // Read runtime code
    // SAFETY: `mmap` is fundamentally unsafe since technically the file can change
    //         underneath us while it is mapped; in practice it's unlikely to be a problem
    let file = std::fs::File::open(runtime).with_context(|| "Failed to open runtime wasm file")?;
    let runtime_code =
        unsafe { memmap2::Mmap::map(&file).with_context(|| "Failed to read runtime wasm file")? };

    // Read raw spec
    let file = std::fs::File::open(raw_spec).with_context(|| "Failed to open raw spec file")?;
    let reader = BufReader::new(file);
    let mut json: serde_json::Value =
        serde_json::from_reader(reader).with_context(|| "Failed to read raw spec file")?;
    println!("json raw specs loaded!");

    let mut hex_runtime_code = String::with_capacity(2 + (runtime_code.len() * 2));
    hex_runtime_code.push('0');
    hex_runtime_code.push('x');
    hex_runtime_code.push_str(&hex::encode(runtime_code));
    //hex::encode_to_slice(runtime_code, &mut hex_runtime_code[2..])
    //.with_context(|| "fail to convert runtime code to hex")?;

    const CODE_KEY: &str = "0x3a636f6465";

    json.as_object_mut()
        .with_context(|| "invalid raw spec file")?
        .get_mut("genesis")
        .with_context(|| "invalid raw spec file: missing field genesis")?
        .as_object_mut()
        .with_context(|| "invalid raw spec file")?
        .get_mut("raw")
        .with_context(|| "invalid raw spec file: missing field raw")?
        .as_object_mut()
        .with_context(|| "invalid raw spec file")?
        .get_mut("top")
        .with_context(|| "invalid raw spec file: missing field top")?
        .as_object_mut()
        .with_context(|| "invalid raw spec file")?
        .insert(
            CODE_KEY.to_owned(),
            serde_json::Value::String(hex_runtime_code),
        );

    // Write modified raw specs

    let file = std::fs::File::create(raw_spec)?;

    serde_json::to_writer_pretty(BufWriter::new(file), &json)
        .with_context(|| "fail to write raw specs")?;

    Ok(())
}

fn build(_production: bool) -> Result<()> {
    exec_should_success(Command::new("cargo").args(["clean", "-p", "duniter"]))?;
    exec_should_success(Command::new("cargo").args(["build", "--locked"]))?;
    exec_should_success(Command::new("mkdir").args(["build"]))?;
    exec_should_success(Command::new("mv").args(["target/debug/duniter", "build/duniter"]))?;

    Ok(())
}

fn test() -> Result<()> {
    exec_should_success(Command::new("cargo").args([
        "test",
        "--workspace",
        "--exclude",
        "duniter-end2end-tests",
    ]))?;

    Ok(())
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
