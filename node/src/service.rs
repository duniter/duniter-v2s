// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

mod client;

pub use sc_executor::NativeExecutor;

use self::client::{Client, RuntimeApiCollection};
use async_io::Timer;
use common_runtime::Block;
use futures::{Stream, StreamExt};
use sc_client_api::{ExecutorProvider, RemoteBackend};
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
use sc_executor::native_executor_instance;
use sc_finality_grandpa::SharedVoterState;
use sc_keystore::LocalKeystore;
use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus::SlotData;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use std::{sync::Arc, time::Duration};

type FullClient<RuntimeApi, Executor> = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

type LightClient<RuntimeApi, Executor> =
    sc_service::TLightClientWithBackend<Block, RuntimeApi, Executor, LightBackend>;
type LightBackend = sc_service::TLightBackendWithHash<Block, sp_runtime::traits::BlakeTwo256>;

native_executor_instance!(
    pub GDevExecutor,
    gdev_runtime::api::dispatch,
    gdev_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

native_executor_instance!(
    pub GTestExecutor,
    gtest_runtime::api::dispatch,
    gtest_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

/// Can be called for a `Configuration` to check if it is a configuration for
/// a particular network.
pub trait IdentifyVariant {
    /// Returns `true` if this is a configuration for the `main` network.
    fn is_main(&self) -> bool;

    /// Returns `true` if this is a configuration for the `test` network.
    fn is_test(&self) -> bool;

    /// Returns `true` if this is a configuration for a dev network.
    fn is_dev(&self) -> bool;
}

impl IdentifyVariant for Box<dyn sc_chain_spec::ChainSpec> {
    fn is_main(&self) -> bool {
        self.id().starts_with("g1")
    }

    fn is_test(&self) -> bool {
        self.id().starts_with("gtest")
    }

    fn is_dev(&self) -> bool {
        self.id().starts_with("dev") || self.id().starts_with("gdev")
    }
}

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
    config: &mut Configuration,
) -> Result<
    (
        Arc<Client>,
        Arc<FullBackend>,
        sp_consensus::import_queue::BasicQueue<Block, sp_trie::PrefixedMemoryDB<BlakeTwo256>>,
        TaskManager,
    ),
    ServiceError,
> {
    if config.chain_spec.is_main() {
        todo!()
    } else if config.chain_spec.is_test() {
        let PartialComponents {
            client,
            backend,
            import_queue,
            task_manager,
            ..
        } = new_partial::<gtest_runtime::RuntimeApi, GTestExecutor>(config, false)?;
        Ok((
            Arc::new(Client::GTest(client)),
            backend,
            import_queue,
            task_manager,
        ))
    } else if config.chain_spec.is_dev() {
        let PartialComponents {
            client,
            backend,
            import_queue,
            task_manager,
            ..
        } = new_partial::<gdev_runtime::RuntimeApi, GDevExecutor>(config, true)?;
        Ok((
            Arc::new(Client::GDev(client)),
            backend,
            import_queue,
            task_manager,
        ))
    } else {
        unreachable!()
    }
}

#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
    consensus_manual: bool,
) -> Result<
    sc_service::PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        FullSelectChain,
        sp_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (
            sc_finality_grandpa::GrandpaBlockImport<
                FullBackend,
                Block,
                FullClient<RuntimeApi, Executor>,
                FullSelectChain,
            >,
            sc_finality_grandpa::LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
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
    if config.keystore_remote.is_some() {
        return Err(ServiceError::Other(
            "Remote Keystores are not supported.".to_owned(),
        ));
    }

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

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
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

    let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let import_queue = if consensus_manual {
        sc_consensus_manual_seal::import_queue(
            Box::new(grandpa_block_import.clone()),
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
        )
    } else {
        let slot_duration = sc_consensus_aura::slot_duration(&*client)?.slot_duration();
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client: client.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        slot_duration,
                    );

                Ok((timestamp, slot))
            },
            spawner: &task_manager.spawn_essential_handle(),
            can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
                client.executor().clone(),
            ),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        })?
    };

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (grandpa_block_import, grandpa_link, telemetry),
    })
}

fn remote_keystore(_url: &str) -> Result<Arc<LocalKeystore>, &'static str> {
    // FIXME: here would the concrete keystore be built,
    //        must return a concrete type (NOT `LocalKeystore`) that
    //        implements `CryptoStore` and `SyncCryptoStore`
    Err("Remote Keystore not supported.")
}

/// Builds a new service for a full client.
pub fn new_full<RuntimeApi, Executor>(
    mut config: Configuration,
    sealing_opt: Option<crate::cli::Sealing>,
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
        mut keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, mut telemetry),
    } = new_partial::<RuntimeApi, Executor>(&config, sealing_opt.is_some())?;

    if let Some(url) = &config.keystore_remote {
        match remote_keystore(url) {
            Ok(k) => keystore_container.set_remote_keystore(k),
            Err(e) => {
                return Err(ServiceError::Other(format!(
                    "Error hooking up remote keystore for {}: {}",
                    url, e
                )))
            }
        };
    }

    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config());

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
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
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        if let Some(sealing) = sealing_opt {
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
                };

            task_manager.spawn_essential_handle().spawn_blocking(
                "authorship_task",
                run_manual_seal(ManualSealParams {
                    block_import,
                    env: proposer_factory,
                    client: client.clone(),
                    pool: transaction_pool.pool().clone(),
                    commands_stream,
                    select_chain,
                    consensus_data_provider: None,
                    create_inherent_data_providers: move |_, ()| async move { Ok(()) },
                }),
            );
        } else {
            let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
            let raw_slot_duration = slot_duration.slot_duration();
            let can_author_with =
                sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

            let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _, _>(
                StartAuraParams {
                    slot_duration,
                    client: client.clone(),
                    select_chain,
                    block_import,
                    proposer_factory,
                    create_inherent_data_providers: move |_, ()| async move {
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        let slot =
                            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                                *timestamp,
                                raw_slot_duration,
                            );

                        Ok((timestamp, slot))
                    },
                    force_authoring,
                    backoff_authoring_blocks,
                    keystore: keystore_container.sync_keystore(),
                    can_author_with,
                    sync_oracle: network.clone(),
                    justification_sync_link: network.clone(),
                    block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
                    max_block_proposal_slot_portion: None,
                    telemetry: telemetry.as_ref().map(|x| x.handle()),
                },
            )?;

            // the AURA authoring task is considered essential, i.e. if it
            // fails we take down the service with it.
            task_manager
                .spawn_essential_handle()
                .spawn_blocking("aura", aura);
        }
    }

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
                command_sink_opt: command_sink_opt.clone(),
            };

            crate::rpc::create_full(deps)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client,
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool,
        rpc_extensions_builder,
        on_demand: None,
        remote_blockchain: None,
        backend,
        system_rpc_tx,
        config,
        telemetry: telemetry.as_mut(),
    })?;

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore_container.sync_keystore())
    } else {
        None
    };

    let grandpa_config = sc_finality_grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_finality_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network,
            voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: SharedVoterState::empty(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();
    Ok(task_manager)
}

/// Builds a new service for a light client.
pub fn new_light<RuntimeApi, Executor>(
    mut config: Configuration,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<LightBackend, Block>>,
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

    let (client, backend, keystore_container, mut task_manager, on_demand) =
        sc_service::new_light_parts::<Block, RuntimeApi, Executor>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;

    let mut telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
        telemetry
    });

    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config());

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
        config.transaction_pool.clone(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
        on_demand.clone(),
    ));

    let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain,
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?.slot_duration();

    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import)),
            client: client.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        slot_duration,
                    );

                Ok((timestamp, slot))
            },
            spawner: &task_manager.spawn_essential_handle(),
            can_author_with: sp_consensus::NeverCanAuthor,
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        })?;

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: Some(on_demand.clone()),
            block_announce_validator_builder: None,
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let enable_grandpa = !config.disable_grandpa;
    if enable_grandpa {
        let name = config.network.node_name.clone();

        let config = sc_finality_grandpa::Config {
            gossip_duration: std::time::Duration::from_millis(333),
            justification_period: 512,
            name: Some(name),
            observer_enabled: false,
            keystore: None,
            local_role: config.role.clone(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        task_manager.spawn_handle().spawn_blocking(
            "grandpa-observer",
            sc_finality_grandpa::run_grandpa_observer(config, grandpa_link, network.clone())?,
        );
    }

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        remote_blockchain: Some(backend.remote_blockchain()),
        transaction_pool,
        task_manager: &mut task_manager,
        on_demand: Some(on_demand),
        rpc_extensions_builder: Box::new(|_, _| ()),
        config,
        client,
        keystore: keystore_container.sync_keystore(),
        backend,
        network,
        system_rpc_tx,
        telemetry: telemetry.as_mut(),
    })?;

    network_starter.start_network();
    Ok(task_manager)
}
