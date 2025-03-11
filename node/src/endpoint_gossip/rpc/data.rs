use crate::endpoint_gossip::rpc::state::PeeringWithId;
use jsonrpsee::core::Serialize;
use serde::Deserialize;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct DuniterPeeringsData {
    pub peerings: Vec<PeeringWithId>,
}
