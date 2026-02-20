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

mod client;
mod gen_doc;
mod gitlab;
mod network;
mod runtime;
mod squid;

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
    /// Release management commands (client, network, runtime)
    Release {
        #[clap(subcommand)]
        command: ReleaseCommand,
    },
    /// Inject runtime code in raw specs
    InjectRuntimeCode {
        #[clap(short, long)]
        /// Runtime filepath
        runtime: PathBuf,
        #[clap(short = 's', long)]
        /// Raw spec filepath
        raw_spec: PathBuf,
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
    NetworkG1Data {
        /// URL du dump G1 à télécharger
        #[clap(long)]
        dump_url: Option<String>,
    },
    /// Build network specs (reprend la tâche build_specs de la CI)
    NetworkBuildSpecs { runtime: String },
    /// Build network runtime (reprend la tâche build_network_runtime de la CI)
    NetworkBuildRuntime { runtime: String },
    /// Create network release (reprend la tâche create_network_release de la CI)
    NetworkCreateRelease {
        /// Nom du réseau (ex: gdev, gtest, g1)
        network: String,
        /// Branche Git à utiliser
        branch: String,
    },
    /// Build raw specs (reprend la tâche build_raw_specs de la CI)
    ClientBuildRawSpecs {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Docker deploy (reprend la tâche docker_deploy de la CI)
    ClientDockerDeploy {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
        /// Architecture cible (amd64, arm64) ou None pour multi-arch
        #[clap(long)]
        arch: Option<String>,
    },
    /// Create client release (reprend la tâche create_client_release de la CI)
    ClientCreateRelease {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
        /// Branche Git à utiliser
        branch: String,
        /// Also upload local DEB/RPM packages to the release
        #[clap(long)]
        upload_packages: bool,
    },
    /// Build RPM (reprend la tâche build_rpm de la CI)
    ClientBuildRpm {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Build DEB (reprend la tâche build_deb de la CI)
    ClientBuildDeb {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
    },
    /// Trigger release builds on GitLab CI and upload artifacts to release
    ClientTriggerReleaseBuilds {
        /// Nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
        network: String,
        /// Branche Git à utiliser
        branch: String,
        /// Tag de la release où uploader les artifacts (optionnel, calculé automatiquement si omis)
        #[clap(long)]
        release_tag: Option<String>,
    },
    /// Build runtime (reprend la tâche build_runtime de la CI)
    RuntimeBuild {
        /// Runtime à construire (gdev, gtest, g1)
        runtime: String,
    },
    /// Create runtime release (reprend la tâche create_runtime_release de la CI)
    RuntimeCreateRelease {
        /// Runtime à publier (gdev, gtest, g1)
        runtime: String,
        /// Branche Git à utiliser
        branch: String,
    },
    /// Trigger squid CI builds (Docker images pushed to Docker Hub)
    SquidTriggerBuilds {
        /// Release tag from duniter-v2s (e.g., g1-1000)
        release_tag: String,
        /// Squid Git branch (default: main)
        #[clap(long, default_value = "main")]
        branch: String,
        /// Custom RPC endpoint for metadata fetching (overrides default per network)
        #[clap(long)]
        rpc_url: Option<String>,
    },
}

#[derive(Debug, clap::Subcommand)]
enum ReleaseCommand {
    /// Client release commands
    #[clap(subcommand)]
    Client(ClientReleaseCommand),
    /// Network release commands
    #[clap(subcommand)]
    Network(NetworkReleaseCommand),
    /// Runtime release commands
    #[clap(subcommand)]
    Runtime(RuntimeReleaseCommand),
    /// Squid indexer release commands
    #[clap(subcommand)]
    Squid(SquidReleaseCommand),
}

#[derive(Debug, clap::Subcommand)]
enum ClientReleaseCommand {
    /// Build raw specs for a network
    BuildRawSpecs {
        /// Format: <network>-<runtime-version> (ex: gtest-1100, g1-1000, gdev-1000)
        network: String,
    },
    /// Create GitLab release with specs files
    Create {
        /// Format: <network>-<runtime-version> (ex: gtest-1100, gdev-1000)
        network: String,
        /// Format: network/<network>-<runtime-version> (ex: network/gtest-1100)
        branch: String,
        /// Also upload local DEB/RPM packages to the release (default: false)
        #[clap(long)]
        upload_packages: bool,
    },
    /// Build DEB package for current architecture
    BuildDeb {
        /// Format: <network>-<runtime-version> (ex: gtest-1100, gdev-1000)
        network: String,
    },
    /// Build RPM package for current architecture
    BuildRpm {
        /// Format: <network>-<runtime-version> (ex: gtest-1100, gdev-1000)
        network: String,
    },
    /// Build and push Docker image (multi-arch by default)
    Docker {
        /// Format: <network>-<runtime-version> (ex: gtest-1100, gdev-1000)
        network: String,
        /// Target architecture: amd64 or arm64 (omit for multi-arch)
        #[clap(long)]
        arch: Option<String>,
    },
    /// Trigger CI builds and upload artifacts to release
    TriggerBuilds {
        /// Format: <network>-<runtime-version> (ex: gtest-1100, gdev-1000)
        network: String,
        /// Format: network/<network>-<runtime-version> (ex: network/gtest-1100)
        branch: String,
        /// Release tag (auto-computed if omitted)
        #[clap(long)]
        release_tag: Option<String>,
    },
}

#[derive(Debug, clap::Subcommand)]
enum NetworkReleaseCommand {
    /// Build network specs for bootstrapping
    BuildSpecs {
        /// Network name (ex: gdev, gtest, g1)
        runtime: String,
    },
    /// Build network runtime WASM for bootstrapping
    BuildRuntime {
        /// Network name (ex: gdev, gtest, g1)
        runtime: String,
    },
    /// Create network release on GitLab
    Create {
        /// Format: <network>-<runtime-version> (ex: gtest-1000, gdev-1000)
        network: String,
        /// Format: network/<network>-<runtime-version> (ex: network/gtest-1000)
        branch: String,
    },
    /// Generate G1 migration data
    G1Data {
        /// Custom G1 dump URL (optional)
        #[clap(long)]
        dump_url: Option<String>,
    },
}

#[derive(Debug, clap::Subcommand)]
enum RuntimeReleaseCommand {
    /// Build runtime WASM binary
    Build {
        /// Runtime name (ex: gdev, gtest, g1)
        runtime: String,
    },
    /// Create runtime release on GitLab
    Create {
        /// Runtime name (ex: gdev, gtest, g1)
        runtime: String,
        /// Format: runtime/<network>-<new-runtime-version> (ex: runtime/gtest-1100)
        branch: String,
    },
}

#[derive(Debug, clap::Subcommand)]
enum SquidReleaseCommand {
    /// Trigger squid CI pipeline to build and push Docker images to Docker Hub
    TriggerBuilds {
        /// Release tag from duniter-v2s (e.g., g1-1000)
        release_tag: String,
        /// Squid Git branch to build from (default: main)
        #[clap(long, default_value = "main")]
        branch: String,
        /// Custom RPC endpoint for metadata fetching (overrides default per network)
        #[clap(long)]
        rpc_url: Option<String>,
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
        DuniterXTaskCommand::PrintSpec { .. }
        | DuniterXTaskCommand::SquidTriggerBuilds { .. }
        | DuniterXTaskCommand::Release {
            command: ReleaseCommand::Squid(_),
        } => {
            /* API-only commands, no Rust toolchain needed */
        }
        _ => {
            Command::new("rustc").arg("--version").status()?;
            Command::new("cargo").arg("--version").status()?;
        }
    }

    match args.command {
        DuniterXTaskCommand::Build { production } => build(production),
        DuniterXTaskCommand::GenDoc => gen_doc::gen_doc(),
        DuniterXTaskCommand::Release { command } => match command {
            ReleaseCommand::Client(cmd) => match cmd {
                ClientReleaseCommand::BuildRawSpecs { network } => {
                    client::build_raw_specs::build_raw_specs(network)
                }
                ClientReleaseCommand::Create {
                    network,
                    branch,
                    upload_packages,
                } => {
                    client::create_client_release::create_client_release(
                        network,
                        branch,
                        upload_packages,
                    )
                    .await
                }
                ClientReleaseCommand::BuildDeb { network } => client::build_deb::build_deb(network),
                ClientReleaseCommand::BuildRpm { network } => client::build_rpm::build_rpm(network),
                ClientReleaseCommand::Docker { network, arch } => {
                    client::docker_deploy::docker_deploy(network, arch)
                }
                ClientReleaseCommand::TriggerBuilds {
                    network,
                    branch,
                    release_tag,
                } => {
                    client::trigger_release_builds::trigger_release_builds(
                        network,
                        branch,
                        release_tag,
                    )
                    .await
                }
            },
            ReleaseCommand::Network(cmd) => match cmd {
                NetworkReleaseCommand::BuildSpecs { runtime } => {
                    network::build_network_specs::build_network_specs(runtime)
                }
                NetworkReleaseCommand::BuildRuntime { runtime } => {
                    network::build_network_runtime::build_network_runtime(runtime)
                }
                NetworkReleaseCommand::Create { network, branch } => {
                    network::create_network_release::create_network_release(network, branch).await
                }
                NetworkReleaseCommand::G1Data { dump_url } => {
                    network::g1_data::g1_data(dump_url).await
                }
            },
            ReleaseCommand::Runtime(cmd) => match cmd {
                RuntimeReleaseCommand::Build { runtime } => {
                    runtime::build_runtime::build_runtime(runtime)
                }
                RuntimeReleaseCommand::Create { runtime, branch } => {
                    runtime::create_runtime_release::create_runtime_release(runtime, branch).await
                }
            },
            ReleaseCommand::Squid(cmd) => match cmd {
                SquidReleaseCommand::TriggerBuilds {
                    release_tag,
                    branch,
                    rpc_url,
                } => {
                    squid::trigger_squid_builds::trigger_squid_builds(
                        release_tag, branch, rpc_url,
                    )
                    .await
                }
            },
        },
        DuniterXTaskCommand::InjectRuntimeCode { runtime, raw_spec } => {
            inject_runtime_code(&raw_spec, &runtime)
        }
        DuniterXTaskCommand::PrintSpec { network } => gitlab::print_spec(network).await,
        DuniterXTaskCommand::CreateAssetLink {
            tag,
            asset_name,
            asset_url,
        } => gitlab::create_asset_link(tag, asset_name, asset_url).await,
        DuniterXTaskCommand::Test => test(),
        DuniterXTaskCommand::NetworkG1Data { dump_url } => {
            network::g1_data::g1_data(dump_url).await
        }
        DuniterXTaskCommand::NetworkBuildSpecs { runtime } => {
            network::build_network_specs::build_network_specs(runtime)
        }
        DuniterXTaskCommand::NetworkBuildRuntime { runtime } => {
            network::build_network_runtime::build_network_runtime(runtime)
        }
        DuniterXTaskCommand::NetworkCreateRelease { network, branch } => {
            network::create_network_release::create_network_release(network, branch).await
        }
        DuniterXTaskCommand::ClientBuildRawSpecs { network } => {
            client::build_raw_specs::build_raw_specs(network)
        }
        DuniterXTaskCommand::ClientDockerDeploy { network, arch } => {
            client::docker_deploy::docker_deploy(network, arch)
        }
        DuniterXTaskCommand::ClientCreateRelease {
            network,
            branch,
            upload_packages,
        } => {
            client::create_client_release::create_client_release(network, branch, upload_packages)
                .await
        }
        DuniterXTaskCommand::ClientBuildRpm { network } => client::build_rpm::build_rpm(network),
        DuniterXTaskCommand::ClientBuildDeb { network } => client::build_deb::build_deb(network),
        DuniterXTaskCommand::ClientTriggerReleaseBuilds {
            network,
            branch,
            release_tag,
        } => {
            client::trigger_release_builds::trigger_release_builds(network, branch, release_tag)
                .await
        }
        DuniterXTaskCommand::RuntimeBuild { runtime } => {
            runtime::build_runtime::build_runtime(runtime)
        }
        DuniterXTaskCommand::RuntimeCreateRelease { runtime, branch } => {
            runtime::create_runtime_release::create_runtime_release(runtime, branch).await
        }
        DuniterXTaskCommand::SquidTriggerBuilds {
            release_tag,
            branch,
            rpc_url,
        } => squid::trigger_squid_builds::trigger_squid_builds(release_tag, branch, rpc_url).await,
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
