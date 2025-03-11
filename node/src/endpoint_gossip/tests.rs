use crate::{
    endpoint_gossip,
    endpoint_gossip::{
        duniter_peering_protocol_name,
        handler::{DuniterPeeringCommand, DuniterPeeringEvent},
        well_known_endpoint_types::RPC,
        DuniterEndpoint, DuniterEndpoints, Peering,
    },
};
use async_channel::Receiver;
use futures::{future, stream, FutureExt, StreamExt};
use log::{debug, warn};
use parking_lot::Mutex;
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, BoxJustificationImport, ImportResult,
    ImportedAux,
};
use sc_network::{NetworkStateInfo, ObservedRole, PeerId};
use sc_network_test::{
    Block, BlockImportAdapter, FullPeerConfig, PassThroughVerifier, PeersClient, TestNetFactory,
};
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedSender};
use sp_api::__private::BlockT;
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::Header;
use std::{future::Future, pin::pin, sync::Arc, task::Poll, time::Duration};

#[tokio::test]
async fn peering_is_forwarded_and_only_once_per_connection() {
    let _ = env_logger::try_init();
    let authorities_count = 3;
    let full_count = 1;
    let total_peers = authorities_count + full_count;
    let mut net = DuniterPeeringTestNet::new(authorities_count, full_count);
    tokio::spawn(start_network(&mut net, total_peers));
    let net = Arc::new(Mutex::new(net));

    // make sure the network is ready (each peering is received by all other peers)
    let wait_for_all_peering_notifications =
        watch_events_and_wait_for_all_peerings(total_peers, &net);
    let wait_for = futures::future::join_all(wait_for_all_peering_notifications).map(|_| ());
    tokio::time::timeout(Duration::from_secs(5), run_until_complete(wait_for, &net))
        .await
        .unwrap();

    // rule: only one peering is accepted per connection (disconnecting/restarting allows to change the peering value)
    let already_received = ensure_only_one_peering_is_accepted(&net);
    tokio::time::timeout(
        Duration::from_secs(5),
        run_until_complete(already_received, &net),
    )
    .await
    .unwrap();
}

fn ensure_only_one_peering_is_accepted(
    net: &Arc<Mutex<DuniterPeeringTestNet>>,
) -> impl Future<Output = ()> {
    let command_0 = net.lock().peer_commands[0].clone();
    let peer_id_0 = net.lock().peer_ids[0];
    let peer_id_1 = net.lock().peer_ids[1];
    let stream_1 = net.lock().peer_streams[1].clone();
    let already_received = async move {
        let mut stream1 = pin!(stream_1);
        while let Some(event) = stream1.next().await {
            if let DuniterPeeringEvent::AlreadyReceivedPeering(peer) = event {
                if peer == peer_id_0 {
                    // We did receive the peering from peer 0
                    break;
                }
            }
        }
    };
    let already_received = futures::future::join_all(vec![already_received]).map(|_| ());
    command_0
        .unbounded_send(DuniterPeeringCommand::SendPeering(
            peer_id_1,
            Peering {
                endpoints: DuniterEndpoints::truncate_from(vec![DuniterEndpoint {
                    protocol: RPC.into(),
                    address: "gdev.example.com:9944".into(),
                }]),
            },
        ))
        .unwrap();
    already_received
}

fn watch_events_and_wait_for_all_peerings(
    total_peers: usize,
    net: &Arc<Mutex<DuniterPeeringTestNet>>,
) -> Vec<impl Future<Output = ()> + Sized> {
    let mut peering_notifications = Vec::new();

    for peer_id in 0..total_peers {
        let local_peer_id = net.lock().peer_ids[peer_id];
        let stream = net.lock().peer_streams[peer_id].clone();
        peering_notifications.push(async move {
            let mut identified = 0;
            let mut stream = pin!(stream);
            while let Some(event) = stream.next().await {
                debug_event(event.clone(), local_peer_id);
                if let DuniterPeeringEvent::GoodPeering(peer, _) = event {
                    debug!(target: "duniter-libp2p", "[{}] Received peering from {}",local_peer_id, peer);
                    identified += 1;
                    if identified == (total_peers - 1) {
                        // all peers identified
                        break;
                    }
                }
            }
            warn!("All peers sent their peering");
        })
    }
    peering_notifications
}

fn debug_event(event: DuniterPeeringEvent, local_peer_id: PeerId) {
    match event {
        DuniterPeeringEvent::StreamOpened(peer, role) => {
            debug!(target: "duniter-libp2p", "[{}] Peer {peer} connected with role {}", local_peer_id, observed_role_to_str(role));
        }
        DuniterPeeringEvent::StreamValidation(peer, result) => {
            debug!(target: "duniter-libp2p", "[{}] Validating inbound substream from {peer} with result {}", local_peer_id, result);
        }
        DuniterPeeringEvent::StreamClosed(peer) => {
            debug!(target: "duniter-libp2p", "[{}] Peer {peer} disconnected", local_peer_id);
        }
        DuniterPeeringEvent::GossipReceived(peer, success) => {
            if success {
                debug!(target: "duniter-libp2p", "[{}] Received peering message from {peer}", local_peer_id);
            } else {
                debug!(target: "duniter-libp2p", "[{}] Failed to receive peering message from {peer}", local_peer_id);
            }
        }
        DuniterPeeringEvent::GoodPeering(peer, _) => {
            debug!(target: "duniter-libp2p", "[{}] Received peering from {}", local_peer_id, peer);
        }
        DuniterPeeringEvent::AlreadyReceivedPeering(peer) => {
            debug!(target: "duniter-libp2p", "[{}] Already received peering from {}", local_peer_id, peer);
            panic!("Received peering from the same peer twice");
        }
        DuniterPeeringEvent::SelfPeeringPropagationFailed(peer, _peering, e) => {
            debug!(target: "duniter-libp2p", "[{}] Failed to propagate self peering to {}: {}", local_peer_id, peer, e);
            panic!("Failed to propagate self peering");
        }
        DuniterPeeringEvent::SelfPeeringPropagationSuccess(peer, _peering) => {
            debug!(target: "duniter-libp2p", "[{}] Successfully propagated self peering to {}", local_peer_id, peer);
        }
    }
}

fn observed_role_to_str(role: ObservedRole) -> &'static str {
    match role {
        ObservedRole::Authority => "Authority",
        ObservedRole::Full => "Full",
        ObservedRole::Light => "Light",
    }
}

// Spawns duniter nodes. Returns a future to spawn on the runtime.
fn start_network(net: &mut DuniterPeeringTestNet, peers: usize) -> impl Future<Output = ()> {
    let nodes = stream::FuturesUnordered::new();

    for peer_id in 0..peers {
        let net_service = net.peers[peer_id].network_service().clone();
        net.peer_ids.push(net_service.local_peer_id());
        let notification_service = net.peers[peer_id]
            .take_notification_service(&format!("/{}", duniter_peering_protocol_name::NAME).into())
            .unwrap();

        let (rpc_sink, mut stream_unbounded) =
            tracing_unbounded("mpsc_duniter_gossip_peering_test", 100_000);
        let (sink_unbounded, stream) = async_channel::unbounded();
        let (command_tx, command_rx) =
            tracing_unbounded("mpsc_duniter_gossip_peering_test_command", 100_000);

        // mapping from mpsc TracingUnboundedReceiver to mpmc Receiver
        tokio::spawn(async move {
            // forward the event
            while let Some(command) = stream_unbounded.next().await {
                sink_unbounded.send(command).await.unwrap();
            }
        });

        let handler = endpoint_gossip::handler::build::<Block, _>(
            notification_service,
            net_service,
            rpc_sink,
            Some(command_rx),
            DuniterEndpoints::new(),
        );
        // To send external commands to the handler (for tests or RPC commands).
        net.peer_streams.push(stream);
        net.peer_commands.push(command_tx);
        let node = handler.run();

        fn assert_send<T: Send>(_: &T) {}
        assert_send(&node);

        nodes.push(node);
    }

    nodes.for_each(|_| async move {})
}

#[derive(Default)]
struct DuniterPeeringTestNet {
    // Peers
    peers: Vec<DuniterPeeringPeer>,
    // IDs of the peers
    peer_ids: Vec<PeerId>,
    // RX of the gossip events
    peer_streams: Vec<Receiver<DuniterPeeringEvent>>,
    // TX to drive the handler (for tests or configuration)
    peer_commands: Vec<TracingUnboundedSender<DuniterPeeringCommand>>,
}

type DuniterPeeringPeer = sc_network_test::Peer<PeerData, DuniterTestBlockImport>;

impl DuniterPeeringTestNet {
    fn new(n_authority: usize, n_full: usize) -> Self {
        let mut net = DuniterPeeringTestNet {
            peers: Vec::with_capacity(n_authority + n_full),
            peer_ids: Vec::new(),
            peer_streams: Vec::new(),
            peer_commands: Vec::new(),
        };

        for _ in 0..n_authority {
            net.add_authority_peer();
        }

        for _ in 0..n_full {
            net.add_full_peer();
        }

        net
    }

    fn add_authority_peer(&mut self) {
        self.add_full_peer_with_config(FullPeerConfig {
            notifications_protocols: vec![
                format!("/{}", duniter_peering_protocol_name::NAME).into()
            ],
            is_authority: true,
            ..Default::default()
        })
    }
}

#[derive(Default)]
struct PeerData;

impl TestNetFactory for DuniterPeeringTestNet {
    type BlockImport = DuniterTestBlockImport;
    type PeerData = PeerData;
    type Verifier = PassThroughVerifier;

    fn make_verifier(&self, _client: PeersClient, _: &PeerData) -> Self::Verifier {
        PassThroughVerifier::new(false) // use non-instant finality.
    }

    fn peer(&mut self, i: usize) -> &mut DuniterPeeringPeer {
        &mut self.peers[i]
    }

    fn peers(&self) -> &Vec<DuniterPeeringPeer> {
        &self.peers
    }

    fn peers_mut(&mut self) -> &mut Vec<DuniterPeeringPeer> {
        &mut self.peers
    }

    fn mut_peers<F: FnOnce(&mut Vec<DuniterPeeringPeer>)>(&mut self, closure: F) {
        closure(&mut self.peers);
    }

    fn make_block_import(
        &self,
        _client: PeersClient,
    ) -> (
        BlockImportAdapter<Self::BlockImport>,
        Option<BoxJustificationImport<Block>>,
        Self::PeerData,
    ) {
        (
            BlockImportAdapter::new(DuniterTestBlockImport),
            None,
            PeerData,
        )
    }

    fn add_full_peer(&mut self) {
        self.add_full_peer_with_config(FullPeerConfig {
            notifications_protocols: vec![
                format!("/{}", duniter_peering_protocol_name::NAME).into()
            ],
            is_authority: false,
            ..Default::default()
        })
    }
}

async fn run_until_complete(future: impl Future + Unpin, net: &Arc<Mutex<DuniterPeeringTestNet>>) {
    let drive_to_completion = futures::future::poll_fn(|cx| {
        net.lock().poll(cx);
        Poll::<()>::Pending
    });
    future::select(future, drive_to_completion).await;
}

#[derive(Clone)]
struct DuniterTestBlockImport;

/// Inspired by GrandpaBlockImport
#[async_trait::async_trait]
impl<Block: BlockT> BlockImport<Block> for DuniterTestBlockImport {
    type Error = ConsensusError;

    /// Fake check block, always succeeds.
    async fn check_block(
        &self,
        _block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        Ok(ImportResult::Imported(ImportedAux {
            is_new_best: true,
            bad_justification: false,
            clear_justification_requests: false,
            header_only: false,
            needs_justification: false,
        }))
    }

    /// Fake import block, always succeeds.
    async fn import_block(
        &self,
        block: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        debug!("Importing block #{}", block.header.number());
        Ok(ImportResult::Imported(ImportedAux {
            is_new_best: true,
            bad_justification: false,
            clear_justification_requests: false,
            header_only: false,
            needs_justification: false,
        }))
    }
}
