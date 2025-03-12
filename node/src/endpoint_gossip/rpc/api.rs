//! # Duniter Peering RPC API
//!
//! Exposes the `duniter_peerings` RPC method.

use crate::endpoint_gossip::rpc::{data::DuniterPeeringsData, state::DuniterPeeringsState};
use jsonrpsee::{core::async_trait, proc_macros::rpc, Extensions};
use sc_consensus_babe_rpc::Error;

/// The exposed RPC methods
#[rpc(client, server)]
pub trait DuniterPeeringRpcApi {
    /// Returns the known peerings list received by network gossips
    #[method(name = "duniter_peerings", with_extensions)]
    async fn duniter_peerings(&self) -> Result<Option<DuniterPeeringsData>, Error>;
}

/// API implementation
pub struct DuniterPeeringRpcApiImpl {
    shared_peer_state: DuniterPeeringsState,
}

impl DuniterPeeringRpcApiImpl {
    /// Creates a new instance of the Duniter Peering Rpc handler.
    pub fn new(shared_peer_state: DuniterPeeringsState) -> Self {
        Self { shared_peer_state }
    }
}

#[async_trait]
impl DuniterPeeringRpcApiServer for DuniterPeeringRpcApiImpl {
    async fn duniter_peerings(
        &self,
        _ext: &Extensions,
    ) -> Result<Option<DuniterPeeringsData>, Error> {
        let option = self.shared_peer_state.peer_state();
        Ok(option)
    }
}
