// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::cli::{Cli, Subcommand};
use crate::service::{GDevExecutor, GTestExecutor, IdentifyVariant};
use crate::{chain_spec, service};
use gdev_runtime::Block;
use sc_cli::{ChainSpec, Role, RuntimeVersion, SubstrateCli};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Substrate Node".into()
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
            "dev" | "gdev" => Box::new(chain_spec::gdev::development_chain_spec()?),
            "gtest_dev" => Box::new(chain_spec::gtest::development_chain_spec()?),
            "local" | "gtest_local" => Box::new(chain_spec::gtest::local_testnet_config()?),
            // Specs provided as json specify which runtime to use in their file name. For example,
            // `g1-custom.json` uses the g1 runtime.
            // `gdev-workshop.json` uses the gdev runtime.
            path => {
                let path = std::path::PathBuf::from(path);

                let starts_with = |prefix: &str| {
                    path.file_name()
                        .map(|f| f.to_str().map(|s| s.starts_with(&prefix)))
                        .flatten()
                        .unwrap_or(false)
                };

                if starts_with("g1") {
                    Box::new(chain_spec::g1::ChainSpec::from_json_file(path)?)
                } else if starts_with("gtest") {
                    Box::new(chain_spec::gtest::ChainSpec::from_json_file(path)?)
                } else if starts_with("gdev") {
                    Box::new(chain_spec::gdev::ChainSpec::from_json_file(path)?)
                } else {
                    panic!("unknown runtime")
                }
            }
        })
    }

    fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        if spec.is_main() {
            todo!() //return &g1_runtime::VERSION;
        } else if spec.is_test() {
            &gtest_runtime::VERSION
        } else if spec.is_dev() {
            &gdev_runtime::VERSION
        } else {
            panic!("unknown runtime")
        }
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                if cmd.shared_params.dev {
                    if config.chain_spec.is_main() {
                        cmd.run(
                            Box::new(chain_spec::g1::development_chain_spec()?),
                            config.network,
                        )
                    } else if config.chain_spec.is_test() {
                        cmd.run(
                            Box::new(chain_spec::gtest::development_chain_spec()?),
                            config.network,
                        )
                    } else if config.chain_spec.is_dev() {
                        cmd.run(
                            Box::new(chain_spec::gdev::development_chain_spec()?),
                            config.network,
                        )
                    } else {
                        panic!("unknown runtime")
                    }
                } else {
                    cmd.run(config.chain_spec, config.network)
                }
            })
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
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
                let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;
                let chain_spec = &runner.config().chain_spec;

                if chain_spec.is_main() {
                    todo!()
                } else if chain_spec.is_test() {
                    runner.sync_run(|config| cmd.run::<Block, GTestExecutor>(config))
                } else if chain_spec.is_dev() {
                    runner.sync_run(|config| cmd.run::<Block, GDevExecutor>(config))
                } else {
                    unreachable!()
                }
            } else {
                Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
                    .into())
            }
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                if config.chain_spec.is_main() {
                    todo!()
                } else if config.chain_spec.is_test() {
                    match config.role {
                        Role::Light => {
                            service::new_light::<gtest_runtime::RuntimeApi, GTestExecutor>(config)
                        }
                        _ => service::new_full::<gtest_runtime::RuntimeApi, GTestExecutor>(
                            config, None,
                        ),
                    }
                    .map_err(sc_cli::Error::Service)
                } else if config.chain_spec.is_dev() {
                    service::new_full::<gdev_runtime::RuntimeApi, GDevExecutor>(
                        config,
                        Some(cli.sealing),
                    )
                    .map_err(sc_cli::Error::Service)
                } else {
                    Err(sc_cli::Error::Application("unknown runtime".into()))
                }
            })
        }
    }
}
