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
use sc_client_api::{client::BlockBackend, Backend};
use sc_consensus_grandpa::{FinalityProofProvider, SharedVoterState};
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::{
    error::Error as ServiceError, Configuration, PartialComponents, TaskManager, WarpSyncConfig,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::TransactionPool;
use sp_consensus_babe::inherents::InherentDataProvider;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use std::{sync::Arc, time::Duration};

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = sp_io::SubstrateHostFunctions;

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions = (
    sp_io::SubstrateHostFunctions,
    frame_benchmarking::benchmarking::HostFunctions,
);

// Allow to use native Runtime for debugging/development purposes
#[cfg(feature = "native")]
type FullClient<RuntimeApi, Executor> =
    sc_service::TFullClient<Block, RuntimeApi, sc_executor::NativeElseWasmExecutor<Executor>>;
// By default, WASM only Runtime
#[cfg(not(feature = "native"))]
type FullClient<RuntimeApi, Executor> =
    sc_service::TFullClient<Block, RuntimeApi, sc_executor::WasmExecutor<Executor>>;

type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub mod runtime_executor {
    use crate::service::HostFunctions;
    #[cfg(feature = "g1")]
    pub use g1_runtime as runtime;
    #[cfg(feature = "gdev")]
    pub use gdev_runtime as runtime;
    #[cfg(feature = "gtest")]
    pub use gtest_runtime as runtime;

    use sc_executor::sp_wasm_interface::{Function, HostFunctionRegistry};

    pub struct Executor;
    impl sc_executor::NativeExecutionDispatch for Executor {
        type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

        fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
            runtime::api::dispatch(method, data)
        }

        fn native_version() -> sc_executor::NativeVersion {
            runtime::native_version()
        }
    }
    impl sc_executor::sp_wasm_interface::HostFunctions for Executor {
        fn host_functions() -> Vec<&'static dyn Function> {
            HostFunctions::host_functions()
        }

        fn register_static<T>(registry: &mut T) -> Result<(), T::Error>
        where
            T: HostFunctionRegistry,
        {
            HostFunctions::register_static(registry)
        }
    }
}
///
/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

#[derive(Debug)]
pub enum RuntimeType {
    G1,
    GDev,
    GTest,
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
        sc_consensus::BasicQueue<Block>,
        TaskManager,
    ),
    ServiceError,
> {
    let PartialComponents {
        client,
        backend,
        import_queue,
        task_manager,
        ..
    } = new_partial::<runtime_executor::runtime::RuntimeApi, runtime_executor::Executor>(
        config,
        manual_consensus,
    )?;
    Ok((
        Arc::new(Client::Client(client)),
        backend,
        import_queue,
        task_manager,
    ))
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
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::TransactionPoolWrapper<Block, FullClient<RuntimeApi, Executor>>,
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
    RuntimeApi::RuntimeApi: RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
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

    #[cfg(feature = "native")]
    let executor = sc_service::new_native_or_wasm_executor(&config);
    #[cfg(not(feature = "native"))]
    let executor = sc_service::new_wasm_executor(&config.executor);

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

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let client_ = client.clone();
    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
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
        let (queue, handle) =
            sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
                link: babe_link.clone(),
                block_import: babe_block_import.clone(),
                justification_import: Some(Box::new(justification_import)),
                client: client.clone(),
                select_chain: select_chain.clone(),
                create_inherent_data_providers: move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot = InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                    Ok((slot, timestamp))
                },
                spawner: &task_manager.spawn_essential_handle(),
                registry: config.prometheus_registry(),
                telemetry: telemetry.as_ref().map(|x| x.handle()),
                offchain_tx_pool_factory:
                    sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
                        transaction_pool.clone(),
                    ),
            })?;

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
pub fn new_full<
    RuntimeApi,
    Executor,
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
    config: Configuration,
    sealing: crate::cli::Sealing,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
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

    let mut net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());
    let metrics = N::register_notification_metrics(config.prometheus_registry());
    let peer_store_handle = net_config.peer_store_handle();
    let (grandpa_protocol_config, grandpa_notification_service) =
        sc_consensus_grandpa::grandpa_peers_set_config::<_, N>(
            grandpa_protocol_name.clone(),
            metrics.clone(),
            peer_store_handle,
        );
    net_config.add_notification_protocol(grandpa_protocol_config);

    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: Some(WarpSyncConfig::WithProvider(warp_sync)),
            block_relay: None,
            metrics,
        })?;

    let role = config.role;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    if config.offchain_worker.enabled {
        use futures::FutureExt;

        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(
                    sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
                        transaction_pool.clone(),
                    ),
                ),
                network_provider: Arc::new(network.clone()),
                is_validator: role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })?
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let mut command_sink_opt = None;
    if role.is_authority() {
        let distance_dir = config
            .base_path
            .config_dir(config.chain_spec.id())
            .join("distance");

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
                            transaction_pool.import_notification_stream().map(|_| {
                                EngineCommand::SealNewBlock {
                                    create_empty: false,
                                    finalize: false,
                                    parent_hash: None,
                                    sender: None,
                                }
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
                            let babe = InherentDataProvider::new(
                                timestamp.slot(),
                            );
                            let distance =
                                dc_distance::create_distance_inherent_data_provider::<
                                    Block,
                                    FullClient<RuntimeApi, Executor>,
                                    FullBackend,
                                >(
                                    &*client, parent, distance_dir, &babe_owner_keys.clone()
                                );
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

                        let slot = InherentDataProvider::from_timestamp_and_slot_duration(
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
                        );

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

    let justification_stream = grandpa_link.justification_stream();
    let shared_authority_set = grandpa_link.shared_authority_set().clone();
    let shared_voter_state = SharedVoterState::empty();
    let finality_proof_provider =
        FinalityProofProvider::new_for_service(backend.clone(), Some(shared_authority_set.clone()));

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain;
        let keystore = keystore_container.keystore().clone();
        let babe_deps = babe_worker_handle.map(|babe_worker_handle| crate::rpc::BabeDeps {
            babe_worker_handle,
            keystore: keystore.clone(),
        });
        let rpc_setup = shared_voter_state.clone();

        Box::new(
            move |subscription_task_executor: SubscriptionTaskExecutor| {
                let grandpa_deps = crate::rpc::GrandpaDeps {
                    shared_voter_state: rpc_setup.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor: subscription_task_executor.clone(),
                    finality_provider: finality_proof_provider.clone(),
                };

                let deps = crate::rpc::FullDeps {
                    client: client.clone(),
                    pool: pool.clone(),
                    select_chain: select_chain.clone(),
                    babe: babe_deps.clone(),
                    grandpa: grandpa_deps,
                    command_sink_opt: command_sink_opt.clone(),
                };

                crate::rpc::create_full(deps).map_err(Into::into)
            },
        )
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend,
        network: network.clone(),
        sync_service: sync_service.clone(),
        client,
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
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
        gossip_duration: Duration::from_millis(333),
        justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
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
            shared_voter_state,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            notification_service: grandpa_notification_service,
            offchain_tx_pool_factory: sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
                transaction_pool.clone(),
            ),
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
        Backend: sc_client_api::Backend<Block>,
        Backend::State: sc_client_api::StateBackend<BlakeTwo256>,
        Api: RuntimeApiCollection,
        Client: client::AbstractClient<Block, Backend, Api = Api> + 'static,
    {
        // Revert consensus-related components.
        // The operations are not correlated, thus call order is not relevant.
        sc_consensus_babe::revert(client.clone(), self.backend, self.blocks)?;
        sc_consensus_grandpa::revert(client, self.blocks)?;
        Ok(())
    }
}
