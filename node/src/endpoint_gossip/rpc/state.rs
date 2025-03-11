use crate::endpoint_gossip::{
    handler::DuniterPeeringEvent, rpc::data::DuniterPeeringsData, DuniterEndpoints,
};
use codec::{Decode, Encode};
use futures::StreamExt;
use jsonrpsee::core::Serialize;
use parking_lot::RwLock;
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedSender};
use serde::Deserialize;
use std::sync::Arc;

/// A struct to hold a peer endpoints along with its id for RPC exposure.
#[derive(Encode, Decode, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PeeringWithId {
    pub peer_id: String,
    pub endpoints: DuniterEndpoints,
}

#[derive(Clone)]
pub struct DuniterPeeringsState {
    inner: Arc<RwLock<Option<Box<DuniterPeeringsData>>>>,
}

/// Dummy CRUD operations for the state to be exposed, plus a listening sink to be notified of
/// network events and automatically insert/remove peers from the state.
impl DuniterPeeringsState {
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Some(Box::new(DuniterPeeringsData {
                peerings: Vec::new(),
            })))),
        }
    }

    pub fn insert(&self, peering: PeeringWithId) -> &Self {
        if let Some(vs) = self.inner.write().as_mut() {
            vs.peerings.push(peering);
        }
        self
    }

    pub fn remove(&self, peer_id: String) -> &Self {
        if let Some(vs) = self.inner.write().as_mut() {
            vs.peerings.retain(|p| p.peer_id != peer_id);
        }
        self
    }

    pub fn peer_state(&self) -> Option<DuniterPeeringsData> {
        self.inner.read().as_ref().map(|vs| vs.as_ref().clone())
    }

    /// Creates a channel for binding to the network events.
    pub fn listen(&self) -> TracingUnboundedSender<DuniterPeeringEvent> {
        let (sink, stream) = tracing_unbounded("mpsc_duniter_peering_rpc_stream", 1_000);
        let state = self.clone();
        tokio::spawn(async move {
            stream
                .for_each(|event| async {
                    match event {
                        DuniterPeeringEvent::GoodPeering(who, peering) => {
                            state.insert(PeeringWithId {
                                peer_id: who.to_base58(),
                                endpoints: peering.endpoints,
                            });
                        }
                        DuniterPeeringEvent::StreamClosed(who) => {
                            state.remove(who.to_base58());
                        }
                        _ => {}
                    }
                })
                .await
        });
        sink
    }
}
