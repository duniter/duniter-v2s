use crate::endpoint_gossip::{
    rpc::{
        api::{DuniterPeeringRpcApiImpl, DuniterPeeringRpcApiServer},
        state::{DuniterPeeringsState, PeeringWithId},
    },
    well_known_endpoint_types::{RPC, SQUID},
    DuniterEndpoint, DuniterEndpoints,
};
use jsonrpsee::RpcModule;

#[tokio::test]
async fn empty_peers_rpc_handler() {
    let rpc = setup_io_handler();
    let expected_response = r#"{"jsonrpc":"2.0","id":0,"result":{"peerings":[]}}"#.to_string();
    let request = r#"{"jsonrpc":"2.0","method":"duniter_peerings","params":[],"id":0}"#;
    let (response, _) = rpc.raw_json_request(request, 1).await.unwrap();

    assert_eq!(expected_response, response);
}

#[tokio::test]
async fn expose_known_peers() {
    let rpc = setup_new_rpc_with_initial_peerings(vec![
        PeeringWithId {
            peer_id: "12D3KooWRkDXunbB64VegYPCQaitcgtdtEtbsbd7f19nsS7aMjDp".into(),
            endpoints: DuniterEndpoints::truncate_from(vec![
                DuniterEndpoint {
                    protocol: RPC.into(),
                    address: "/rpc/wss://gdev.example.com".into(),
                },
                DuniterEndpoint {
                    protocol: SQUID.into(),
                    address: "/squid/https://squid.gdev.gyroi.de/v1/graphql".into(),
                },
            ]),
        },
        PeeringWithId {
            peer_id: "12D3KooWFiUBo3Kjiryvrpz8b3kfNVk7baezhab7SHdfafgY7nmN".into(),
            endpoints: DuniterEndpoints::truncate_from(vec![DuniterEndpoint {
                protocol: RPC.into(),
                address: "/rpc/ws://gdev.example.com:9944".into(),
            }]),
        },
    ]);
    let expected_response = r#"{"jsonrpc":"2.0","id":0,"result":{"peerings":[{"peer_id":"12D3KooWRkDXunbB64VegYPCQaitcgtdtEtbsbd7f19nsS7aMjDp","endpoints":[{"protocol":"rpc","address":"/rpc/wss://gdev.example.com"},{"protocol":"squid","address":"/squid/https://squid.gdev.gyroi.de/v1/graphql"}]},{"peer_id":"12D3KooWFiUBo3Kjiryvrpz8b3kfNVk7baezhab7SHdfafgY7nmN","endpoints":[{"protocol":"rpc","address":"/rpc/ws://gdev.example.com:9944"}]}]}}"#.to_string();
    let request = r#"{"jsonrpc":"2.0","method":"duniter_peerings","params":[],"id":0}"#;
    let (response, _) = rpc.raw_json_request(request, 1).await.unwrap();

    assert_eq!(expected_response, response);
}

fn setup_io_handler() -> RpcModule<DuniterPeeringRpcApiImpl> {
    DuniterPeeringRpcApiImpl::new(DuniterPeeringsState::empty()).into_rpc()
}

fn setup_new_rpc_with_initial_peerings(
    peers: Vec<PeeringWithId>,
) -> RpcModule<DuniterPeeringRpcApiImpl> {
    let state = DuniterPeeringsState::empty();
    for peer in peers {
        state.insert(peer);
    }
    DuniterPeeringRpcApiImpl::new(state).into_rpc()
}
