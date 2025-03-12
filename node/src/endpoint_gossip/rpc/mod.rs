//! # RPC for peering
//!
//! This module gathers all known peering documents for connected peers in memory and provides
//! an RPC interface to query them.
//!
//! ## RPC methods
//!
//! Currently, only one RPC method is available to query the currently known peerings.
//! In the future, the RPC interface could add methods to dynamically change the current node's peering
//! without restarting the node.
//!
//! ### `duniter_peerings`
//!
//! Returns the known peerings list received by network gossips.
//!
//! ```json
//! {
//!     "jsonrpc": "2.0",
//!     "id": 0,
//!     "result": {
//!         "peers": [
//!             {
//!                 "endpoints": [
//!                     "/rpc/wss://gdev.example.com",
//!                     "/squid/https://squid.gdev.gyroi.de/v1/graphql"
//!                 ]
//!             },
//!             {
//!                 "endpoints": [
//!                     "/rpc/ws://gdev.example.com:9944"
//!                 ]
//!             }
//!         ]
//!     }
//! }
//! ```
//!
pub mod api;
pub mod data;
pub mod state;

#[cfg(test)]
mod tests;
