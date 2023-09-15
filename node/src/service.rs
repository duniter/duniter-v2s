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

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

pub mod client;

use self::client::{Client, ClientHandle, RuntimeApiCollection};
use async_io::Timer;
use common_runtime::Block;
use futures::{Stream, StreamExt};
use sc_client_api::client::BlockBackend;
use sc_consensus_grandpa::SharedVoterState;
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
pub use sc_executor::NativeElseWasmExecutor;
use sc_network_common::sync::warp::WarpSyncParams;
use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use std::{path::PathBuf, sync::Arc, time::Duration};

type FullClient<RuntimeApi, Executor> =
    sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

#[cfg(feature = "gdev")]
pub struct GDevExecutor;
#[cfg(feature = "gdev")]
impl sc_executor::NativeExecutionDispatch for GDevExecutor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        gdev_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        gdev_runtime::native_version()
    }
}

#[cfg(feature = "gtest")]
pub struct GTestExecutor;
#[cfg(feature = "gtest")]
impl sc_executor::NativeExecutionDispatch for GTestExecutor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        gtest_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        gtest_runtime::native_version()
    }
}

#[cfg(feature = "g1")]
pub struct G1Executor;
#[cfg(feature = "g1")]
impl sc_executor::NativeExecutionDispatch for G1Executor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        g1_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        g1_runtime::native_version()
    }
}

#[derive(Debug)]
pub enum RuntimeType {
    G1,
    GDev,
    GTest,
}

/// Can be called for a `Configuration` to check if it is a configuration for
/// a particular runtime type.
pub trait IdentifyRuntimeType {
    /// Returns the runtime type
    fn runtime_type(&self) -> RuntimeType;
}

impl IdentifyRuntimeType for Box<dyn sc_chain_spec::ChainSpec> {
    fn runtime_type(&self) -> RuntimeType {
        if self.id().starts_with("g1") {
            RuntimeType::G1
        } else if self.id().starts_with("dev") || self.id().starts_with("gdev") {
            RuntimeType::GDev
        } else if self.id().starts_with("gtest") {
            RuntimeType::GTest
        } else {
            panic!("unknown runtime")
        }
    }
}

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
    config: &Configuration,
    manual_consensus: bool,
) -> Result<
    (
        Arc<Client>,
        Arc<FullBackend>,
        sc_consensus::BasicQueue<Block, sp_trie::PrefixedMemoryDB<BlakeTwo256>>,
        TaskManager,
    ),
    ServiceError,
> {
    match config.chain_spec.runtime_type() {
        #[cfg(feature = "g1")]
        RuntimeType::G1::G1 => {
            let PartialComponents {
                client,
                backend,
                import_queue,
                task_manager,
                ..
            } = new_partial::<g1_runtime::RuntimeApi, G1Executor>(config, manual_consensus)?;
            Ok((
                Arc::new(Client::G1(client)),
                backend,
                import_queue,
                task_manager,
            ))
        }
        #[cfg(feature = "gtest")]
        RuntimeType::GTest => {
            let PartialComponents {
                client,
                backend,
                import_queue,
                task_manager,
                ..
            } = new_partial::<gtest_runtime::RuntimeApi, GTestExecutor>(config, manual_consensus)?;
            Ok((
                Arc::new(Client::GTest(client)),
                backend,
                import_queue,
                task_manager,
            ))
        }
        #[cfg(feature = "gdev")]
        RuntimeType::GDev => {
            let PartialComponents {
                client,
                backend,
                import_queue,
                task_manager,
                ..
            } = new_partial::<gdev_runtime::RuntimeApi, GDevExecutor>(config, manual_consensus)?;
            Ok((
                Arc::new(Client::GDev(client)),
                backend,
                import_queue,
                task_manager,
            ))
        }
        _ => panic!("unknown runtime"),
    }
}

type FullGrandpaBlockImport<RuntimeApi, Executor> = sc_consensus_grandpa::GrandpaBlockImport<
    FullBackend,
    Block,
    FullClient<RuntimeApi, Executor>,
    FullSelectChain,
>;

#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
    consensus_manual: bool,
) -> Result<
    sc_service::PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (
            sc_consensus_babe::BabeBlockImport<
                Block,
                FullClient<RuntimeApi, Executor>,
                FullGrandpaBlockImport<RuntimeApi, Executor>,
            >,
            sc_consensus_babe::BabeLink<Block>,
            Option<sc_consensus_babe::BabeWorkerHandle<Block>>,
            sc_consensus_grandpa::LinkHalf<
                Block,
                FullClient<RuntimeApi, Executor>,
                FullSelectChain,
            >,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_native_or_wasm_executor(config);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let client_ = client.clone();
    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        &(client_ as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let justification_import = grandpa_block_import.clone();

    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::configuration(&*client)?,
        grandpa_block_import,
        client.clone(),
    )?;

    let (import_queue, babe_worker_handle) = if consensus_manual {
        let import_queue = sc_consensus_manual_seal::import_queue(
            Box::new(babe_block_import.clone()),
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
        );
        (import_queue, None)
    } else {
        let slot_duration = babe_link.config().slot_duration();
        let (queue, handle) = sc_consensus_babe::import_queue(
            babe_link.clone(),
            babe_block_import.clone(),
            Some(Box::new(justification_import)),
            client.clone(),
            select_chain.clone(),
            move |_parent, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );
                Ok((slot, timestamp))
            },
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
            telemetry.as_ref().map(|x| x.handle()),
        )?;
        (queue, Some(handle))
    };

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (
            babe_block_import,
            babe_link,
            babe_worker_handle,
            grandpa_link,
            telemetry,
        ),
    })
}

/// Builds a new service for a full client.
pub fn new_full<RuntimeApi, Executor>(
    mut config: Configuration,
    sealing: crate::cli::Sealing,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, babe_link, babe_worker_handle, grandpa_link, mut telemetry),
    } = new_partial::<RuntimeApi, Executor>(&config, sealing.is_manual_consensus())?;

    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    config
        .network
        .extra_sets
        .push(sc_consensus_grandpa::grandpa_peers_set_config(
            grandpa_protocol_name.clone(),
        ));
    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let mut command_sink_opt = None;
    if role.is_authority() {
        let distance_dir = config.base_path.as_ref().map_or_else(
            || {
                PathBuf::from(format!(
                    "/tmp/duniter/chains/{}/distance",
                    config.chain_spec.id()
                ))
            },
            |base_path| {
                base_path
                    .config_dir(config.chain_spec.id())
                    .join("distance")
            },
        );

        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let keystore_ptr = keystore_container.keystore();
        let client = client.clone();

        if sealing.is_manual_consensus() {
            let commands_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> =
                match sealing {
                    crate::cli::Sealing::Instant => {
                        Box::new(
                            // This bit cribbed from the implementation of instant seal.
                            transaction_pool
                                .pool()
                                .validated_pool()
                                .import_notification_stream()
                                .map(|_| EngineCommand::SealNewBlock {
                                    create_empty: false,
                                    finalize: false,
                                    parent_hash: None,
                                    sender: None,
                                }),
                        )
                    }
                    crate::cli::Sealing::Manual => {
                        let (sink, stream) = futures::channel::mpsc::channel(1000);
                        // Keep a reference to the other end of the channel. It goes to the RPC.
                        command_sink_opt = Some(sink);
                        Box::new(stream)
                    }
                    crate::cli::Sealing::Interval(millis) => Box::new(StreamExt::map(
                        Timer::interval(Duration::from_millis(millis)),
                        |_| EngineCommand::SealNewBlock {
                            create_empty: true,
                            finalize: false,
                            parent_hash: None,
                            sender: None,
                        },
                    )),
                    crate::cli::Sealing::Production => unreachable!(),
                };

            let babe_consensus_data_provider =
                sc_consensus_manual_seal::consensus::babe::BabeConsensusDataProvider::new(
                    client.clone(),
                    keystore_container.keystore(),
                    babe_link.epoch_changes().clone(),
                    vec![(
                        sp_consensus_babe::AuthorityId::from(
                            sp_keyring::sr25519::Keyring::Alice.public(),
                        ),
                        1000,
                    )],
                )
                .expect("failed to create BabeConsensusDataProvider");

            task_manager.spawn_essential_handle().spawn_blocking(
                "manual-seal",
                Some("block-authoring"),
                run_manual_seal(ManualSealParams {
                    block_import,
                    env: proposer_factory,
                    client: client.clone(),
                    pool: transaction_pool.clone(),
                    commands_stream,
                    select_chain: select_chain.clone(),
                    consensus_data_provider: Some(Box::new(babe_consensus_data_provider)),
                    create_inherent_data_providers: move |parent, _| {
                        let client = client.clone();
                        let distance_dir = distance_dir.clone();
                        let babe_owner_keys =
                            std::sync::Arc::new(sp_keystore::Keystore::sr25519_public_keys(
                                keystore_ptr.as_ref(),
                                sp_runtime::KeyTypeId(*b"babe"),
                            ));
                        async move {
                            let timestamp =
                                sc_consensus_manual_seal::consensus::timestamp::SlotTimestampProvider::new_babe(
                                    client.clone(),
                                )
                                .map_err(|err| format!("{:?}", err))?;
                            let babe = sp_consensus_babe::inherents::InherentDataProvider::new(
                                timestamp.slot(),
                            );
                            let distance =
                                dc_distance::create_distance_inherent_data_provider::<
                                    Block,
                                    FullClient<RuntimeApi, Executor>,
                                    FullBackend,
                                >(
                                    &*client, parent, distance_dir, &babe_owner_keys.clone()
                                )?;
                            Ok((timestamp, babe, distance))
                        }
                    },
                }),
            );
        } else {
            let slot_duration = babe_link.config().slot_duration();
            let babe_config = sc_consensus_babe::BabeParams {
                keystore: keystore_container.keystore(),
                client: client.clone(),
                select_chain: select_chain.clone(),
                block_import,
                env: proposer_factory,
                sync_oracle: sync_service.clone(),
                justification_sync_link: sync_service.clone(),
                create_inherent_data_providers: move |parent, ()| {
                    // This closure is called during each block generation.

                    let client = client.clone();
                    let distance_dir = distance_dir.clone();
                    let babe_owner_keys =
                        std::sync::Arc::new(sp_keystore::Keystore::sr25519_public_keys(
                            keystore_ptr.as_ref(),
                            sp_runtime::KeyTypeId(*b"babe"),
                        ));

                    async move {
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        let slot =
						sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

                        let storage_proof =
                            sp_transaction_storage_proof::registration::new_data_provider(
                                &*client, &parent,
                            )?;

                        let distance = dc_distance::create_distance_inherent_data_provider::<
                            Block,
                            FullClient<RuntimeApi, Executor>,
                            FullBackend,
                        >(
                            &*client, parent, distance_dir, &babe_owner_keys.clone()
                        )?;

                        Ok((slot, timestamp, storage_proof, distance))
                    }
                },
                force_authoring,
                backoff_authoring_blocks,
                babe_link,
                block_proposal_slot_portion: sc_consensus_babe::SlotProportion::new(2f32 / 3f32),
                max_block_proposal_slot_portion: None,
                telemetry: telemetry.as_ref().map(|x| x.handle()),
            };
            let babe = sc_consensus_babe::start_babe(babe_config)?;

            // the BABE authoring task is considered essential, i.e. if it
            // fails we take down the service with it.
            task_manager.spawn_essential_handle().spawn_blocking(
                "babe-proposer",
                Some("block-authoring"),
                babe,
            );
        }
    }

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain;
        let chain_spec = config.chain_spec.cloned_box();
        let keystore = keystore_container.keystore().clone();
        let babe_deps = babe_worker_handle.map(|babe_worker_handle| crate::rpc::BabeDeps {
            babe_worker_handle,
            keystore: keystore.clone(),
        });

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: select_chain.clone(),
                chain_spec: chain_spec.cloned_box(),
                deny_unsafe,
                babe: babe_deps.clone(),
                command_sink_opt: command_sink_opt.clone(),
            };

            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend,
        network: network.clone(),
        sync_service: sync_service.clone(),
        client,
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool,
        rpc_builder: rpc_extensions_builder,
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore_container.keystore())
    } else {
        None
    };

    let grandpa_config = sc_consensus_grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        protocol_name: grandpa_protocol_name,
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_consensus_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            sync: sync_service,
            network,
            voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: SharedVoterState::empty(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();

    log::info!("***** Duniter has fully started *****");

    Ok(task_manager)
}

/// Reverts the node state down to at most the last finalized block.
///
/// In particular this reverts:
/// - Low level Babe and Grandpa consensus data.
pub fn revert_backend(
    client: Arc<Client>,
    backend: Arc<FullBackend>,
    blocks: common_runtime::BlockNumber,
) -> sc_cli::Result<()> {
    // Revert Substrate consensus related components
    client.execute_with(RevertConsensus { blocks, backend })?;
    Ok(())
}

pub(super) struct RevertConsensus {
    blocks: common_runtime::BlockNumber,
    backend: Arc<FullBackend>,
}

impl client::ExecuteWithClient for RevertConsensus {
    type Output = sp_blockchain::Result<()>;

    fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
    where
        <Api as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        Backend: sc_client_api::Backend<Block> + 'static,
        Backend::State: sp_api::StateBackend<BlakeTwo256>,
        Api: RuntimeApiCollection<StateBackend = Backend::State>,
        Client: client::AbstractClient<Block, Backend, Api = Api> + 'static,
    {
        // Revert consensus-related components.
        // The operations are not correlated, thus call order is not relevant.
        sc_consensus_babe::revert(client.clone(), self.backend, self.blocks)?;
        sc_consensus_grandpa::revert(client, self.blocks)?;
        Ok(())
    }
}
