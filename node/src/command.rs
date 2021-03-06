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

pub mod key;
pub mod utils;

use crate::cli::{Cli, Subcommand};
#[cfg(feature = "g1")]
use crate::service::G1Executor;
#[cfg(feature = "gdev")]
use crate::service::GDevExecutor;
#[cfg(feature = "gtest")]
use crate::service::GTestExecutor;
use crate::service::{IdentifyRuntimeType, RuntimeType};
use crate::{chain_spec, service};
use clap::CommandFactory;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};

// TODO: create our own reference hardware
/*
lazy_static! {
    /// The hardware requirements as measured on reference hardware.
    pub static ref REFERENCE_HARDWARE: Requirements = {
        let raw = include_bytes!("reference_hardware.json").as_slice();
        serde_json::from_slice(raw).expect("Hardcoded data is known good; qed")
    };
}*/

/// Unwraps a [`crate::client::Client`] into the concrete runtime client.
macro_rules! unwrap_client {
    (
		$client:ident,
		$code:expr
	) => {
        match $client.as_ref() {
            #[cfg(feature = "g1")]
            crate::service::client::Client::G1($client) => $code,
            #[cfg(feature = "gtest")]
            crate::service::client::Client::GTest($client) => $code,
            #[cfg(feature = "gdev")]
            crate::service::client::Client::GDev($client) => $code,
            #[allow(unreachable_patterns)]
            _ => panic!("unknown runtime"),
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
        "support.anonymous.an".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            #[cfg(feature = "gdev")]
            "dev" => Box::new(chain_spec::gdev::development_chain_spec()?),
            #[cfg(feature = "gdev")]
            "local" | "gdev_local" => Box::new(chain_spec::gdev::local_testnet_config(1, 3, 4)?),
            #[cfg(feature = "gdev")]
            "local2" => Box::new(chain_spec::gdev::local_testnet_config(2, 3, 4)?),
            #[cfg(feature = "gdev")]
            "local3" => Box::new(chain_spec::gdev::local_testnet_config(3, 3, 4)?),
            #[cfg(feature = "gdev")]
            "local4" => Box::new(chain_spec::gdev::local_testnet_config(4, 4, 5)?),
            #[cfg(feature = "gdev")]
            "gdev-gl" | "gdev_gl" => Box::new(chain_spec::gdev::gen_live_conf()?),
            #[cfg(feature = "gdev")]
            "gdev" => Box::new(chain_spec::gdev::ChainSpec::from_json_bytes(
                &include_bytes!("../specs/gdev-raw.json")[..],
            )?),
            #[cfg(feature = "gtest")]
            "gtest_dev" => Box::new(chain_spec::gtest::development_chain_spec()?),
            #[cfg(feature = "gtest")]
            "gtest_local" => Box::new(chain_spec::gtest::local_testnet_config(2, 3)?),
            #[cfg(feature = "gtest")]
            "gtest_local3" => Box::new(chain_spec::gtest::local_testnet_config(3, 4)?),
            #[cfg(feature = "gtest")]
            "gtest_local4" => Box::new(chain_spec::gtest::local_testnet_config(4, 5)?),
            #[cfg(feature = "gtest")]
            "gtest" => {
                unimplemented!()
                //Box::new(chain_spec::gtest::ChainSpec::from_json_file(file_path)?)
            }
            #[cfg(feature = "g1")]
            "g1" => {
                unimplemented!()
                //Box::new(chain_spec::g1::ChainSpec::from_json_file(file_path)?)
            }
            // Specs provided as json specify which runtime to use in their file name. For example,
            // `g1-custom.json` uses the g1 runtime.
            // `gdev-workshop.json` uses the gdev runtime.
            path => {
                let path = std::path::PathBuf::from(path);

                let starts_with = |prefix: &str| {
                    path.file_name()
                        .and_then(|f| f.to_str().map(|s| s.starts_with(&prefix)))
                        .unwrap_or(false)
                };

                let runtime_type = if starts_with("g1") {
                    RuntimeType::G1
                } else if starts_with("gdem") {
                    RuntimeType::GTest
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

    fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        match spec.runtime_type() {
            #[cfg(feature = "g1")]
            RuntimeType::G1 => &g1_runtime::VERSION,
            #[cfg(feature = "gtest")]
            RuntimeType::GTest => &gtest_runtime::VERSION,
            #[cfg(feature = "gdev")]
            RuntimeType::GDev => &gdev_runtime::VERSION,
            _ => panic!("unknown runtime"),
        }
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
            runner.sync_run(|config| {
                if cmd.shared_params.dev {
                    match config.chain_spec.runtime_type() {
                        #[cfg(feature = "g1")]
                        RuntimeType::G1 => cmd.run(
                            Box::new(chain_spec::g1::development_chain_spec()?),
                            config.network,
                        ),
                        #[cfg(feature = "gtest")]
                        RuntimeType::GTest => cmd.run(
                            Box::new(chain_spec::gtest::development_chain_spec()?),
                            config.network,
                        ),
                        #[cfg(feature = "gdev")]
                        RuntimeType::GDev => cmd.run(
                            Box::new(chain_spec::gdev::development_chain_spec()?),
                            config.network,
                        ),
                        _ => panic!("unknown runtime"),
                    }
                } else {
                    cmd.run(config.chain_spec, config.network)
                }
            })
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) =
                    service::new_chain_ops(&mut config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) =
                    service::new_chain_ops(&mut config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) =
                    service::new_chain_ops(&mut config, cli.sealing.is_manual_consensus())?;
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
                    service::new_chain_ops(&mut config, cli.sealing.is_manual_consensus())?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, backend, _, task_manager) =
                    service::new_chain_ops(&mut config, cli.sealing.is_manual_consensus())?;
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
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            match cmd {
                BenchmarkCmd::Storage(cmd) => runner.sync_run(|mut config| {
                    let (client, backend, _, _) = service::new_chain_ops(&mut config, false)?;
                    let db = backend.expose_db();
                    let storage = backend.expose_storage();

                    unwrap_client!(client, cmd.run(config, client.clone(), db, storage))
                }),
                BenchmarkCmd::Block(cmd) => runner.sync_run(|mut config| {
                    let (client, _, _, _) = service::new_chain_ops(&mut config, false)?;

                    unwrap_client!(client, cmd.run(client.clone()))
                }),
                BenchmarkCmd::Overhead(cmd) => runner.sync_run(|mut config| {
                    let (client, _, _, _) = service::new_chain_ops(&mut config, false)?;
                    let wrapped = client.clone();

                    let inherent_data = crate::service::client::benchmark_inherent_data()
                        .map_err(|e| format!("generating inherent data: {:?}", e))?;

                    unwrap_client!(
                        client,
                        cmd.run(config, client.clone(), inherent_data, wrapped)
                    )
                }),
                BenchmarkCmd::Pallet(cmd) => {
                    if cfg!(feature = "runtime-benchmarks") {
                        match chain_spec.runtime_type() {
                            #[cfg(feature = "g1")]
                            RuntimeType::G1 => runner.sync_run(|config| {
                                cmd.run::<g1_runtime::Block, G1Executor>(config)
                            }),
                            #[cfg(feature = "gtest")]
                            RuntimeType::GTest => runner.sync_run(|config| {
                                cmd.run::<gtest_runtime::Block, GTestExecutor>(config)
                            }),
                            #[cfg(feature = "gdev")]
                            RuntimeType::GDev => runner.sync_run(|config| {
                                cmd.run::<gdev_runtime::Block, GDevExecutor>(config)
                            }),
                            _ => Err(sc_cli::Error::Application("unknown runtime type".into())),
                        }
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
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            use sc_service::TaskManager;
            let registry = &runner
                .config()
                .prometheus_config
                .as_ref()
                .map(|cfg| &cfg.registry);
            let task_manager = TaskManager::new(runner.config().tokio_handle.clone(), *registry)
                .map_err(|e| {
                    sc_cli::Error::Application(format!("Fail to create TaskManager: {}", e).into())
                })?;

            // Ensure dev spec
            if !chain_spec.id().ends_with("dev") {
                return Err(sc_cli::Error::Application(
                    "try-runtime only support dev specs".into(),
                ));
            }

            match chain_spec.runtime_type() {
                #[cfg(feature = "gdev")]
                RuntimeType::GDev => {
                    //sp_core::crypto::set_default_ss58_version(Ss58AddressFormatRegistry::GDev);
                    runner.async_run(|config| {
                        Ok((
                            cmd.run::<gdev_runtime::Block, GDevExecutor>(config),
                            task_manager,
                        ))
                    })
                }
                _ => Err(sc_cli::Error::Application("unknown runtime type".into())),
            }
        }
        #[cfg(not(feature = "try-runtime"))]
        Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
            .into()),
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|mut config| async move {
                // Force offchain worker and offchain indexing if we have the role Authority
                if config.role.is_authority() {
                    config.offchain_worker.enabled = true;
                    config.offchain_worker.indexing_enabled = true;
                }

                match config.chain_spec.runtime_type() {
                    #[cfg(feature = "g1")]
                    RuntimeType::G1 => {
                        service::new_full::<g1_runtime::RuntimeApi, G1Executor>(config, cli.sealing)
                            .map_err(sc_cli::Error::Service)
                    }
                    #[cfg(feature = "gtest")]
                    RuntimeType::GTest => service::new_full::<
                        gtest_runtime::RuntimeApi,
                        GTestExecutor,
                    >(config, cli.sealing)
                    .map_err(sc_cli::Error::Service),
                    #[cfg(feature = "gdev")]
                    RuntimeType::GDev => {
                        service::new_full::<gdev_runtime::RuntimeApi, GDevExecutor>(
                            config,
                            cli.sealing,
                        )
                        .map_err(sc_cli::Error::Service)
                    }
                    _ => Err(sc_cli::Error::Application("unknown runtime".into())),
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
