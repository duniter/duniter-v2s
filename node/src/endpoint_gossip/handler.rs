use crate::endpoint_gossip::{
    types::validation_result::DuniterStreamValidationResult, DuniterEndpoints, Peer, Peering,
    PROPAGATE_TIMEOUT,
};
use codec::{Decode, Encode};
use futures::{stream, FutureExt, Stream, StreamExt};
use log::debug;
use sc_network::{
    service::traits::{NotificationEvent, ValidationResult},
    utils::interval,
    NetworkEventStream, NetworkPeers, NetworkStateInfo, NotificationService, ObservedRole, PeerId,
};
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use sp_api::__private::BlockT;
use std::{collections::HashMap, marker::PhantomData, pin::Pin};

pub fn build<
    B: BlockT + 'static,
    N: NetworkPeers + NetworkEventStream + NetworkStateInfo + Clone,
>(
    notification_service: Box<dyn NotificationService>,
    network: N,
    rpc_sink: TracingUnboundedSender<DuniterPeeringEvent>,
    command_rx: Option<TracingUnboundedReceiver<DuniterPeeringCommand>>,
    endpoints: DuniterEndpoints,
) -> GossipsHandler<B, N> {
    let local_peer_id = network.local_peer_id();

    GossipsHandler {
        b: PhantomData::<B>,
        notification_service,
        propagate_timeout: (Box::pin(interval(PROPAGATE_TIMEOUT))
            as Pin<Box<dyn Stream<Item = ()> + Send>>)
            .fuse(),
        network,
        peers: HashMap::new(),
        command_rx: command_rx.unwrap_or_else(|| {
            let (_tx, rx) = tracing_unbounded("mpsc_duniter_peering_rpc_command", 1_000);
            rx
        }),
        self_peering: Peering { endpoints },
        events_reporter: DuniterEventsReporter {
            sink: rpc_sink,
            local_peer_id,
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DuniterPeeringEvent {
    StreamOpened(PeerId, ObservedRole),
    StreamValidation(PeerId, DuniterStreamValidationResult),
    StreamClosed(PeerId),
    /// Received gossip from a peer, `bool` indicates whether the gossip was successfully decoded.
    GossipReceived(PeerId, bool),
    GoodPeering(PeerId, Peering),
    AlreadyReceivedPeering(PeerId),
    SelfPeeringPropagationSuccess(PeerId, Peering),
    SelfPeeringPropagationFailed(PeerId, Peering, String),
}

pub enum DuniterPeeringCommand {
    /// Send a peering to a peer.
    #[allow(dead_code)] // only used in tests for now, maybe in the future by RPC
    SendPeering(PeerId, Peering),
}

struct DuniterEventsReporter {
    sink: TracingUnboundedSender<DuniterPeeringEvent>,
    local_peer_id: PeerId,
}

impl DuniterEventsReporter {
    /// Report an event for monitoring purposes (logs + unit tests).
    fn report_event(&self, event: DuniterPeeringEvent) {
        self.sink.unbounded_send(event.clone())
            .unwrap_or_else(|e| {
                log::error!(target: "duniter-libp2p", "[{}] Failed to send notification: {}", self.local_peer_id, e);
            })
    }
}

/// Handler for gossips. Call [`GossipsHandler::run`] to start the processing.
pub struct GossipsHandler<
    B: BlockT + 'static,
    N: NetworkPeers + NetworkEventStream + NetworkStateInfo,
> {
    b: PhantomData<B>,
    /// Interval at which we try to propagate our peering
    propagate_timeout: stream::Fuse<Pin<Box<dyn Stream<Item = ()> + Send>>>,
    /// Network service to use to send messages and manage peers.
    network: N,
    /// All connected peers and their known peering.
    peers: HashMap<PeerId, Peer>,
    /// The interal peering of the node.
    self_peering: Peering,
    /// Internal sink to report events.
    events_reporter: DuniterEventsReporter,
    /// Receiver for external commands (tests/RPC methods).
    command_rx: TracingUnboundedReceiver<DuniterPeeringCommand>,
    /// Handle that is used to communicate with `sc_network::Notifications`.
    notification_service: Box<dyn NotificationService>,
}

impl<B, N> GossipsHandler<B, N>
where
    B: BlockT + 'static,
    N: NetworkPeers + NetworkEventStream + NetworkStateInfo,
{
    /// Turns the [`TransactionsHandler`] into a future that should run forever and not be
    /// interrupted.
    pub async fn run(mut self) {
        // Share self peering do listeners of current handler
        self.events_reporter
            .report_event(DuniterPeeringEvent::GoodPeering(
                self.network.local_peer_id(),
                self.self_peering.clone(),
            ));
        // Then start the network loop
        loop {
            futures::select! {
                _ = self.propagate_timeout.next() => {
                    for (peer, peer_data) in self.peers.iter_mut() {
                        if !peer_data.sent_peering {
                            match self.notification_service.send_async_notification(peer, self.self_peering.encode()).await {
                                Ok(_) => {
                                    peer_data.sent_peering = true;
                                    self.events_reporter.report_event(DuniterPeeringEvent::SelfPeeringPropagationSuccess(*peer, self.self_peering.clone()));
                                }
                                Err(e) => {
                                    self.events_reporter.report_event(DuniterPeeringEvent::SelfPeeringPropagationFailed(*peer, self.self_peering.clone(), e.to_string()));
                                }
                            }
                        }
                    }
                },
                command = self.command_rx.next().fuse() => {
                    if let Some(command) = command {
                        self.handle_command(command).await
                    }
                },
                event = self.notification_service.next_event().fuse() => {
                    if let Some(event) = event {
                        self.handle_notification_event(event)
                    } else {
                        // `Notifications` has seemingly closed. Closing as well.
                        return
                    }
                }
            }
        }
    }

    fn handle_notification_event(&mut self, event: NotificationEvent) {
        match event {
            NotificationEvent::ValidateInboundSubstream {
                peer,
                handshake,
                result_tx,
                ..
            } => {
                // only accept peers whose role can be determined
                let result = self
                    .network
                    .peer_role(peer, handshake)
                    .map_or(ValidationResult::Reject, |_| ValidationResult::Accept);
                let duniter_validation = DuniterStreamValidationResult::from(result);
                self.events_reporter
                    .report_event(DuniterPeeringEvent::StreamValidation(
                        peer,
                        duniter_validation.clone(),
                    ));
                let _ = result_tx.send(duniter_validation.into());
            }
            NotificationEvent::NotificationStreamOpened {
                peer, handshake, ..
            } => {
                let Some(role) = self.network.peer_role(peer, handshake) else {
                    debug!(target: "duniter-libp2p", "[{}] role for {peer} couldn't be determined", self.network.local_peer_id());
                    return;
                };

                let _was_in = self.peers.insert(
                    peer,
                    Peer {
                        sent_peering: false,
                        known_peering: None,
                    },
                );
                debug_assert!(_was_in.is_none());
                self.events_reporter
                    .report_event(DuniterPeeringEvent::StreamOpened(peer, role));
            }
            NotificationEvent::NotificationStreamClosed { peer } => {
                let _peer = self.peers.remove(&peer);
                debug_assert!(_peer.is_some());
                self.events_reporter
                    .report_event(DuniterPeeringEvent::StreamClosed(peer));
            }
            NotificationEvent::NotificationReceived { peer, notification } => {
                if let Ok(peering) = <Peering as Decode>::decode(&mut notification.as_ref()) {
                    self.events_reporter
                        .report_event(DuniterPeeringEvent::GossipReceived(peer, true));
                    self.on_peering(peer, peering);
                } else {
                    self.events_reporter
                        .report_event(DuniterPeeringEvent::GossipReceived(peer, false));
                    self.network.report_peer(peer, rep::BAD_PEERING);
                }
            }
        }
    }

    /// Called when peer sends us new peerings
    fn on_peering(&mut self, who: PeerId, peering: Peering) {
        if let Some(ref mut peer) = self.peers.get_mut(&who) {
            if peer.known_peering.is_some() {
                // Peering has already been received for this peer. Only one is allowed per connection.
                self.network.report_peer(who, rep::BAD_PEERING);
                self.events_reporter
                    .report_event(DuniterPeeringEvent::AlreadyReceivedPeering(who));
            } else {
                peer.known_peering = Some(peering.clone());
                self.events_reporter
                    .report_event(DuniterPeeringEvent::GoodPeering(who, peering.clone()));
                self.network.report_peer(who, rep::GOOD_PEERING);
            }
        }
    }

    async fn handle_command(&mut self, cmd: DuniterPeeringCommand) {
        match cmd {
            DuniterPeeringCommand::SendPeering(peer, peering) => {
                debug!(target: "duniter-libp2p", "[{}]Sending COMMANDED self peering to {}", self.network.local_peer_id(), peer);
                match self
                    .notification_service
                    .send_async_notification(&peer, peering.encode())
                    .await
                {
                    Ok(_) => {
                        self.events_reporter.report_event(
                            DuniterPeeringEvent::SelfPeeringPropagationSuccess(peer, peering),
                        );
                    }
                    Err(e) => {
                        self.events_reporter.report_event(
                            DuniterPeeringEvent::SelfPeeringPropagationFailed(
                                peer,
                                peering,
                                e.to_string(),
                            ),
                        );
                    }
                }
            }
        };
    }
}

mod rep {
    use sc_network::ReputationChange as Rep;
    /// Reputation change when a peer sends us an peering that we didn't know about.
    pub const GOOD_PEERING: Rep = Rep::new(1 << 7, "Good peering");
    /// Reputation change when a peer sends us a bad peering.
    pub const BAD_PEERING: Rep = Rep::new(-(1 << 12), "Bad peering");
}
