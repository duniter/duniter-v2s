// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

pub mod key;
pub mod utils;

use crate::{
    chain_spec,
    cli::{Cli, DuniterConfigExtension, Subcommand},
    service,
    service::{RuntimeType, runtime_executor::Executor},
};
use clap::CommandFactory;
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use sc_cli::SubstrateCli;
#[cfg(feature = "runtime-benchmarks")]
use sc_executor::{NativeExecutionDispatch, sp_wasm_interface::ExtendedHostFunctions};

// TODO: create our own reference hardware
/*
lazy_static! {
    /// The hardware requirements as measured on reference hardware.
    pub static ref REFERENCE_HARDWARE: Requirements = {
        let raw = include_bytes!("reference_hardware.json").as_slice();
        serde_json::from_slice(raw).expect("Hardcoded data is known good; qed")
    };
}*/

/// Unwraps a [`crate::service::client::Client`] into the concrete runtime client.
#[cfg(feature = "runtime-benchmarks")]
macro_rules! unwrap_client {
    ($client:ident, $code:expr) => {
        match $client.as_ref() {
            crate::service::client::Client::Client($client) => $code,
        }
    };
}

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Duniter".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://forum.duniter.org/".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            // Development chainspec with generated genesis and Alice as a validator
            // For benchmarking, the total length of identities should be at least MinReceivedCertToBeAbleToIssueCert + 1
            #[cfg(feature = "gdev")]
            "dev" => Box::new(chain_spec::gdev::local_testnet_config(1, 5, 6)?),

            // Local testnet with G1 data, Gdev configuration (parameters & Smiths), and Alice as a validator.
            // Optionally, load configuration from DUNITER_GENESIS_CONFIG file to override default Gdev configuration.
            #[cfg(all(feature = "gdev", not(feature = "embed")))]
            "gdev" => Box::new(chain_spec::gdev::gdev_development_chain_spec(
                "resources/gdev.yaml".to_string(),
            )?),

            // Generate development chainspecs with Alice as a validator.
            // Provide the DUNITER_GENESIS_CONFIG environment variable to build genesis from JSON; otherwise, a local testnet with generated genesis will be used.
            #[cfg(all(feature = "gdev", not(feature = "embed")))]
            "gdev_dev" => Box::new(chain_spec::gdev::gdev_development_chain_spec(
                "resources/gdev.yaml".to_string(),
            )?),

            // Chainspecs for live network with G1 data, Gdev configuration (parameters & Smiths).
            // A Smith with declared session keys is required.
            // Optionally load configuration from DUNITER_GENESIS_CONFIG file to override default Gdev configuration.
            #[cfg(feature = "gdev")]
            "gdev_live" => {
                const CLIENT_SPEC: &str = "./node/specs/gdev_client-specs.yaml";
                let client_spec: chain_spec::gdev::ClientSpec = serde_yaml::from_slice(
                    &std::fs::read(
                        std::env::var("DUNITER_CLIENT_SPEC")
                            .unwrap_or_else(|_| CLIENT_SPEC.to_string()),
                    )
                    .map_err(|e| format!("failed to read {CLIENT_SPEC} {e}"))?[..],
                )
                .map_err(|e| format!("failed to parse {e}"))?;
                Box::new(chain_spec::gdev::gen_live_conf(
                    client_spec,
                    "resources/gdev.yaml".to_string(),
                )?)
            }

            // Hardcoded raw chainspecs with previously generated values, resulting in a needlessly heavy binary due to hexadecimal-text-encoded values.
            #[cfg(all(feature = "gdev", feature = "embed"))]
            "gdev" => Box::new(chain_spec::gdev::ChainSpec::from_json_bytes(
                &include_bytes!("../specs/gdev-raw.json")[..],
            )?),

            // For benchmarking, the total length of identities should be at least MinReceivedCertToBeAbleToIssueCert + 1
            #[cfg(feature = "gtest")]
            "dev" => Box::new(chain_spec::gtest::local_testnet_config(1, 5, 6)?),

            // Generate development chainspecs with Alice as a validator.
            // Provide the DUNITER_GTEST_GENESIS environment variable to build genesis from JSON; otherwise, a local testnet with generated genesis will be used.
            #[cfg(feature = "gtest")]
            "gtest_dev" => Box::new(chain_spec::gtest::development_chainspecs(
                "resources/gtest.yaml".to_string(),
            )?),

            // Chainspecs for the live network.
            // Required files in the ./node/specs folder or override with environment variables:
            // - gtest.json / DUNITER_GTEST_GENESIS
            // - gtest_client-specs.json / DUNITER_GTEST_CLIENT_SPEC
            #[cfg(feature = "gtest")]
            "gtest_live" => {
                const JSON_CLIENT_SPEC: &str = "./node/specs/gtest_client-specs.yaml";
                let client_spec: chain_spec::gtest::ClientSpec = serde_yaml::from_slice(
                    &std::fs::read(
                        std::env::var("DUNITER_CLIENT_SPEC")
                            .unwrap_or_else(|_| JSON_CLIENT_SPEC.to_string()),
                    )
                    .map_err(|e| format!("failed to read {JSON_CLIENT_SPEC} {e}"))?[..],
                )
                .map_err(|e| format!("failed to parse {e}"))?;
                Box::new(chain_spec::gtest::live_chainspecs(
                    client_spec,
                    "resources/gtest.yaml".to_string(),
                )?)
            }

            // Return hardcoded live chainspecs, only with the embed feature enabled.
            // Embed client spec and genesis to avoid embedding hexadecimal runtime
            // and having hexadecimal runtime in the git history.
            // This will only build on a branch that has a file named ./node/specs/gtest-raw.json.
            #[cfg(all(feature = "gtest", feature = "embed"))]
            "gtest" => Box::new(chain_spec::gtest::ChainSpec::from_json_bytes(
                &include_bytes!("../specs/gtest-raw.json")[..],
            )?),

            // For benchmarking, the total length of identities should be at least MinReceivedCertToBeAbleToIssueCert + 1
            #[cfg(feature = "g1")]
            "dev" => Box::new(chain_spec::g1::local_testnet_config(1, 5, 6)?),

            path => {
                let path = std::path::PathBuf::from(path);

                let starts_with = |prefix: &str| {
                    path.file_name()
                        .and_then(|f| f.to_str().map(|s| s.starts_with(prefix)))
                        .unwrap_or(false)
                };

                let runtime_type = if starts_with("g1") {
                    RuntimeType::G1
                } else if starts_with("dev") || starts_with("gdev") {
                    RuntimeType::GDev
                } else if starts_with("gt") {
                    RuntimeType::GTest
                } else {
                    panic!("unknown runtime")
                };

                match runtime_type {
                    #[cfg(feature = "g1")]
                    RuntimeType::G1 => Box::new(chain_spec::g1::ChainSpec::from_json_file(path)?),
                    #[cfg(feature = "gdev")]
                    RuntimeType::GDev => {
                        Box::new(chain_spec::gdev::ChainSpec::from_json_file(path)?)
                    }
                    #[cfg(feature = "gtest")]
                    RuntimeType::GTest => {
                        Box::new(chain_spec::gtest::ChainSpec::from_json_file(path)?)
                    }
                    _ => panic!("unknown runtime"),
                }
            }
        })
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let mut cli = Cli::from_args();

    // Force some cli options
    force_cli_options(&mut cli);

    match &cli.subcommand {
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let (client, _, import_queue, task_manager) =
                    service::new_chain_ops(&config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let (client, _, _, task_manager) =
                    service::new_chain_ops(&config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let (client, _, _, task_manager) =
                    service::new_chain_ops(&config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                // Force offchain worker and offchain indexing if we have the role Authority
                if config.role.is_authority() {
                    config.offchain_worker.enabled = true;
                    config.offchain_worker.indexing_enabled = true;
                }

                let (client, _, import_queue, task_manager) =
                    service::new_chain_ops(&config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let (client, backend, _, task_manager) =
                    service::new_chain_ops(&config, cli.sealing.is_manual_consensus())?;
                let aux_revert = Box::new(|client, backend, blocks| {
                    service::revert_backend(client, backend, blocks)
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Utils(cmd)) => cmd.run(&cli),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Completion(cmd)) => {
            let command = &mut Cli::command();
            clap_complete::generate(
                cmd.generator,
                command,
                command.get_name().to_string(),
                &mut std::io::stdout(),
            );
            Ok(())
        }
        #[cfg(feature = "distance-oracle")]
        Some(Subcommand::DistanceOracle(cmd)) => sc_cli::build_runtime()?.block_on(async move {
            let mut builder = sc_cli::LoggerBuilder::new("");
            builder.with_profiling(sc_cli::TracingReceiver::Log.into(), cmd.log.clone());
            builder.init()?;
            let client = distance_oracle::api::client(&cmd.rpc_url).await;

            let settings = distance_oracle::Settings {
                evaluation_result_dir: cmd.evaluation_result_dir.clone().into(),
                rpc_url: cmd.rpc_url.clone(),
            };

            if let Some(duration) = cmd.interval {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(duration));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    distance_oracle::run(&client, &settings).await;
                    interval.tick().await;
                }
            } else {
                distance_oracle::run(&client, &settings).await;
            }
            Ok(())
        }),
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(&**cmd)?;
            let _chain_spec = &runner.config().chain_spec;

            match &**cmd {
                BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
                    let (client, backend, ..) = service::new_chain_ops(&config, false)?;
                    let db = backend.expose_db();
                    let storage = backend.expose_storage();
                    let shared_cache = backend.expose_shared_trie_cache();

                    unwrap_client!(
                        client,
                        cmd.run(config, client.clone(), db, storage, shared_cache)
                    )
                }),
                BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
                    let (client, _, _, _) = service::new_chain_ops(&config, false)?;

                    unwrap_client!(client, cmd.run(client.clone()))
                }),
                BenchmarkCmd::Overhead(cmd) => runner.sync_run(|config| {
                    let (client, _, _, _) = service::new_chain_ops(&config, false)?;
                    let wrapped = client.clone();

                    let inherent_data = crate::service::client::benchmark_inherent_data()
                        .map_err(|e| format!("generating inherent data: {:?}", e))?;

                    unwrap_client!(
                        client,
                        cmd.run(
                            config.chain_spec.name().into(),
                            client.clone(),
                            inherent_data,
                            Vec::new(),
                            wrapped.as_ref(),
                            false,
                        )
                    )
                }),
                BenchmarkCmd::Pallet(cmd) => {
                    if cfg!(feature = "runtime-benchmarks") {
                        runner.sync_run(|config| {
                            cmd.run_with_spec::<sp_runtime::traits::HashingFor<
                                service::runtime_executor::runtime::Block,
                            >, ExtendedHostFunctions<
                                sp_io::SubstrateHostFunctions,
                                <Executor as NativeExecutionDispatch>::ExtendHostFunctions,
                            >>(Some(config.chain_spec))
                        })
                    } else {
                        Err("Benchmarking wasn't enabled when building the node. \
								You can enable it with `--features runtime-benchmarks`."
                            .into())
                    }
                }
                BenchmarkCmd::Machine(cmd) => {
                    runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
                }
                // NOTE: this allows the Duniter client to leniently implement
                // new benchmark commands.
                #[allow(unreachable_patterns)]
                _ => panic!("unknown runtime"),
            }
        }
        #[cfg(not(feature = "runtime-benchmarks"))]
        Some(Subcommand::Benchmark(_cmd)) => {
            Err("Benchmark wasn't enabled when building the node. \
            You can enable it with `--features runtime-benchmarks`."
                .into())
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            let duniter_options: DuniterConfigExtension = cli.duniter_options;
            runner.run_node_until_exit(|mut config| async move {
                // Force offchain worker and offchain indexing if we have the role Authority
                if config.role.is_authority() {
                    config.offchain_worker.enabled = true;
                    config.offchain_worker.indexing_enabled = true;
                }

                {
                    service::new_full::<
                        service::runtime_executor::runtime::RuntimeApi,
                        Executor,
                        sc_network::Litep2pNetworkBackend,
                    >(config, cli.sealing, duniter_options)
                    .map_err(sc_cli::Error::Service)
                }
            })
        }
    }
}

fn force_cli_options(cli: &mut Cli) {
    match cli.subcommand {
        Some(Subcommand::CheckBlock(ref mut cmd)) => {
            cmd.import_params.database_params.database = Some(sc_cli::Database::ParityDb);
        }
        Some(Subcommand::ExportBlocks(ref mut cmd)) => {
            cmd.database_params.database = Some(sc_cli::Database::ParityDb);
        }
        Some(Subcommand::ImportBlocks(ref mut cmd)) => {
            cmd.import_params.database_params.database = Some(sc_cli::Database::ParityDb);
        }
        Some(Subcommand::PurgeChain(ref mut cmd)) => {
            cmd.database_params.database = Some(sc_cli::Database::ParityDb);
        }
        None => {
            cli.run.import_params.database_params.database = Some(sc_cli::Database::ParityDb);
        }
        _ => {}
    }
}
