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

//! A collection of node-specific RPC methods.
//!
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use crate::endpoint_gossip::rpc::{api::DuniterPeeringRpcApiServer, state::DuniterPeeringsState};
use common_runtime::{AccountId, Balance, Block, BlockNumber, Hash, Index};
use jsonrpsee::RpcModule;
use sc_consensus_babe::{BabeApi, BabeWorkerHandle};
use sc_consensus_grandpa::{
    self, FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_keystore::KeystorePtr;
use std::sync::Arc;

/// Extra dependencies for BABE.
#[derive(Clone)]
pub struct BabeDeps {
    /// A handle to the BABE worker for issuing requests.
    pub babe_worker_handle: BabeWorkerHandle<Block>,
    /// The keystore that manages the keys of the node.
    pub keystore: KeystorePtr,
}

/// Dependencies for GRANDPA
#[derive(Clone)]
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Dependencies for DuniterPeering
#[derive(Clone)]
pub struct DuniterPeeringRpcModuleDeps {
    /// The state of the DuniterPeering RPC module which will be exposed.
    pub state: DuniterPeeringsState,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// Manual seal command sink
    pub command_sink_opt: Option<
        futures::channel::mpsc::Sender<sc_consensus_manual_seal::EngineCommand<sp_core::H256>>,
    >,
    /// BABE specific dependencies.
    pub babe: Option<BabeDeps>,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
    /// DuniterPeering specific dependencies.
    pub duniter_peering: DuniterPeeringRpcModuleDeps,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, SC, B>(
    deps: FullDeps<C, P, SC, B>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use sc_consensus_babe_rpc::{Babe, BabeApiServer};
    use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
    use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        select_chain,
        command_sink_opt,
        babe,
        grandpa,
        duniter_peering: endpoint_gossip,
    } = deps;

    if let Some(babe) = babe {
        let BabeDeps {
            babe_worker_handle,
            keystore,
        } = babe;
        module.merge(
            Babe::new(client.clone(), babe_worker_handle, keystore, select_chain).into_rpc(),
        )?;
    }

    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;
    module.merge(
        Grandpa::new(
            subscription_executor,
            shared_authority_set,
            shared_voter_state,
            justification_stream,
            finality_provider,
        )
        .into_rpc(),
    )?;

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    if let Some(command_sink) = command_sink_opt {
        // We provide the rpc handler with the sending end of the channel to allow the rpc
        // send EngineCommands to the background block authorship task.
        module.merge(ManualSeal::new(command_sink).into_rpc())?;
    };

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`
    module.merge(
        crate::endpoint_gossip::rpc::api::DuniterPeeringRpcApiImpl::new(endpoint_gossip.state)
            .into_rpc(),
    )?;

    Ok(module)
}
