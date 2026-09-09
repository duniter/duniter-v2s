pub(crate) mod handler;
pub(crate) mod rpc;
#[cfg(test)]
mod tests;
mod types;

use crate::endpoint_gossip::duniter_peering_protocol_name::NAME;
use codec::{Decode, DecodeAll, Encode};
use sc_network::{
    NetworkBackend, NotificationMetrics, NotificationService,
    config::{PeerStoreProvider, SetConfig},
    types::ProtocolName,
};
use serde::{Deserialize, Serialize};
use sp_api::__private::BlockT;
use sp_core::{ConstU32, bounded_vec::BoundedVec};
use std::{sync::Arc, time};

pub mod well_known_endpoint_types {
    pub const RPC: &str = "rpc";
    pub const SQUID: &str = "squid";
}

pub struct DuniterPeeringParams {
    /// Handle that is used to communicate with `sc_network::Notifications`.
    pub notification_service: Box<dyn NotificationService>,
}

/// Maximum allowed size for a Duniter peering notification.
pub(crate) const MAX_GOSSIP_SIZE: u64 = 32 * 1024;
pub(crate) const MAX_ENDPOINT_PROTOCOL_SIZE: usize = 64;
pub(crate) const MAX_ENDPOINT_ADDRESS_SIZE: usize = 2 * 1024;

/// Interval at which we propagate gossips;
pub(crate) const PROPAGATE_TIMEOUT: time::Duration = time::Duration::from_secs(1);

pub mod duniter_peering_protocol_name {

    pub(crate) const NAME: &str = "duniter-peerings/1";
}

impl DuniterPeeringParams {
    /// Create a new instance.
    pub fn new<
        Hash: AsRef<[u8]>,
        Block: BlockT,
        Net: NetworkBackend<Block, <Block as BlockT>::Hash>,
    >(
        genesis_hash: Hash,
        fork_id: Option<&str>,
        metrics: NotificationMetrics,
        peer_store_handle: Arc<dyn PeerStoreProvider>,
    ) -> (Self, Net::NotificationProtocolConfig) {
        let genesis_hash = genesis_hash.as_ref();
        let protocol_name: ProtocolName = if let Some(fork_id) = fork_id {
            format!(
                "/{}/{}/{}",
                array_bytes::bytes2hex("", genesis_hash),
                fork_id,
                NAME,
            )
        } else {
            format!("/{}/{}", array_bytes::bytes2hex("", genesis_hash), NAME)
        }
        .into();
        let (config, notification_service) = Net::notification_config(
            protocol_name.clone(),
            vec![format!("/{}/{}", array_bytes::bytes2hex("", genesis_hash), NAME).into()],
            MAX_GOSSIP_SIZE,
            None,
            // Default config, allowing some non-reserved nodes to connect
            SetConfig::default(),
            metrics,
            peer_store_handle,
        );

        (
            Self {
                notification_service,
            },
            config,
        )
    }
}

/// Peer information
#[derive(Debug)]
struct Peer {
    /// Whether this peer already sent its peering during the current connection.
    known_peering: bool,
    sent_peering: bool,
}

#[derive(Encode, Decode, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DuniterEndpoint {
    /// The name of the endpoint (e.g. "rpc" or "squid") are well-known names
    pub protocol: String,
    /// The endpoint itself (e.g. "squid.example.com/v1/graphql")
    pub address: String,
}

pub type DuniterEndpoints = BoundedVec<DuniterEndpoint, ConstU32<10>>;

#[derive(Encode, Decode, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Peering {
    pub endpoints: DuniterEndpoints,
}

impl Peering {
    fn decode_and_validate(mut input: &[u8]) -> Option<Self> {
        let peering = Self::decode_all(&mut input).ok()?;
        peering.has_valid_field_lengths().then_some(peering)
    }

    fn has_valid_field_lengths(&self) -> bool {
        self.endpoints.iter().all(|endpoint| {
            endpoint.protocol.len() <= MAX_ENDPOINT_PROTOCOL_SIZE
                && endpoint.address.len() <= MAX_ENDPOINT_ADDRESS_SIZE
        })
    }
}
