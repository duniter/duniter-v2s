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

mod build_deb;
mod build_network_runtime;
mod build_network_specs;
mod build_raw_specs;
mod build_rpm;
mod build_runtime;
mod create_client_release;
mod create_network_release;
mod docker_deploy;
mod g1_data;
mod gen_doc;
mod gitlab;

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
        /// Name of the release + tag to be applied
        name: String,
        /// Name of the network to be put in the release notes title of the srtool part
        network: String,
        /// Branch on which the tag `name` will be created during the release
        branch: String,
        /// Name of the milestone to add this release to
        milestone: String,
    },
    /// Release a new client for a network
    ReleaseClient {
        /// Name of the release + tag to be applied
        name: String,
        /// Branch on which the tag `name` will be created during the release
        branch: String,
        /// Name of the milestone to add this release to
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
    /// Generate G1 data using Docker and py-g1-migrator
    G1Data {
        /// URL du dump G1 à télécharger
        #[clap(long)]
        dump_url: String,
    },
    /// Build network specs (reprend la tâche build_specs de la CI)
    BuildNetworkSpecs {
        /// Runtime à utiliser (gdev, gtest, g1)
        #[clap(long, default_value = "gdev")]
        runtime: String,
    },
    /// Build network runtime (reprend la tâche build_network_runtime de la CI)
    BuildNetworkRuntime {
        /// Runtime à utiliser (gdev, gtest, g1)
        #[clap(long, default_value = "gdev")]
        runtime: String,
    },
    /// Create network release (reprend la tâche create_network_release de la CI)
    CreateNetworkRelease {
        /// Nom du réseau (ex: gdev, gtest, g1)
        network: String,
        /// Branche Git à utiliser
        branch: String,
    },
    /// Build raw specs (reprend la tâche build_raw_specs de la CI)
    BuildRawSpecs {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Docker deploy (reprend la tâche docker_deploy de la CI)
    DockerDeploy {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Create client release (reprend la tâche create_client_release de la CI)
    CreateClientRelease {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
        /// Branche Git à utiliser
        branch: String,
    },
    /// Build RPM (reprend la tâche build_rpm de la CI)
    BuildRpm {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Build DEB (reprend la tâche build_deb de la CI)
    BuildDeb {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Build runtime (reprend la tâche build_runtime de la CI)
    BuildRuntime {
        /// Runtime à construire (gdev, gtest, g1)
        runtime: String,
    },
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
            gitlab::release_network(network, branch).await
        }
        DuniterXTaskCommand::ReleaseRuntime {
            name,
            network,
            branch,
            milestone,
        } => gitlab::release_runtime(name, network, branch, milestone).await,
        DuniterXTaskCommand::ReleaseClient {
            name,
            branch,
            milestone,
        } => gitlab::release_client(name, branch, milestone).await,
        DuniterXTaskCommand::PrintSpec { network } => gitlab::print_spec(network).await,
        DuniterXTaskCommand::CreateAssetLink {
            tag,
            asset_name,
            asset_url,
        } => gitlab::create_asset_link(tag, asset_name, asset_url).await,
        DuniterXTaskCommand::Test => test(),
        DuniterXTaskCommand::G1Data { dump_url } => g1_data::g1_data(dump_url).await,
        DuniterXTaskCommand::BuildNetworkSpecs { runtime } => {
            build_network_specs::build_network_specs(runtime)
        }
        DuniterXTaskCommand::BuildNetworkRuntime { runtime } => {
            build_network_runtime::build_network_runtime(runtime)
        }
        DuniterXTaskCommand::CreateNetworkRelease { network, branch } => {
            create_network_release::create_network_release(network, branch).await
        }
        DuniterXTaskCommand::BuildRawSpecs { network } => build_raw_specs::build_raw_specs(network),
        DuniterXTaskCommand::DockerDeploy { network } => docker_deploy::docker_deploy(network),
        DuniterXTaskCommand::CreateClientRelease { network, branch } => {
            create_client_release::create_client_release(network, branch).await
        }
        DuniterXTaskCommand::BuildRpm { network } => build_rpm::build_rpm(network),
        DuniterXTaskCommand::BuildDeb { network } => build_deb::build_deb(network),
        DuniterXTaskCommand::BuildRuntime { runtime } => build_runtime::build_runtime(runtime),
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
