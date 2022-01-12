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

#![allow(clippy::enum_variant_names)]

pub mod balances;

#[subxt::subxt(runtime_metadata_path = "../resources/metadata.scale")]
pub mod node_runtime {}

use serde_json::Value;
use sp_keyring::AccountKeyring;
use std::process::Command;
use subxt::{ClientBuilder, DefaultConfig, DefaultExtra};

pub type Api = node_runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;
pub type Client = subxt::Client<DefaultConfig>;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const SUDO_ACCOUNT: AccountKeyring = AccountKeyring::Alice;

pub struct Process(std::process::Child);

impl Drop for Process {
    fn drop(&mut self) {
        self.0.kill().expect("node already down");
    }
}

pub async fn spawn_node() -> (Api, Client, Process) {
    let p2p_port = portpicker::pick_unused_port().expect("No ports free");
    let rpc_port = portpicker::pick_unused_port().expect("No ports free");
    let ws_port = portpicker::pick_unused_port().expect("No ports free");
    let process = Process(
        Command::new("../target/debug/duniter")
            .args([
                "--execution=Native",
                "--no-telemetry",
                "--no-prometheus",
                "--dev",
                "--sealing=manual",
                "--tmp",
                "--port",
                &p2p_port.to_string(),
                "--rpc-port",
                &rpc_port.to_string(),
                "--ws-port",
                &ws_port.to_string(),
            ])
            .spawn()
            .expect("failed to spawn node"),
    );
    let duration_secs = if let Ok(duration_string) =
        std::env::var("DUNITER_INTEGRATION_TESTS_SPAWN_NODE_DURATION")
    {
        duration_string.parse().unwrap_or(4)
    } else {
        4
    };
    std::thread::sleep(std::time::Duration::from_secs(duration_secs));

    let client = ClientBuilder::new()
        .set_url(format!("ws://127.0.0.1:{}", ws_port))
        .build()
        .await
        .expect("fail to connect to node");
    let api = client.clone().to_runtime_api::<Api>();

    (api, client, process)
}

/*pub async fn create_empty_block(client: &Client) -> Result<(), subxt::Error> {
    // Create an empty block
    let _: Value = client
        .rpc()
        .client
        .request(
            "engine_createBlock",
            &[Value::Bool(true), Value::Bool(false), Value::Null],
        )
        .await?;

    Ok(())
}*/

pub async fn create_block_with_extrinsic(
    client: &Client,
    extrinsic: subxt::UncheckedExtrinsic<DefaultConfig, DefaultExtra<DefaultConfig>>,
) -> Result<subxt::TransactionEvents<DefaultConfig>> {
    // Get a hash of the extrinsic (we'll need this later).
    use subxt::sp_runtime::traits::Hash as _;
    let ext_hash = <DefaultConfig as subxt::Config>::Hashing::hash_of(&extrinsic);
    // Submit and watch for transaction progress.
    let sub = client.rpc().watch_extrinsic(extrinsic).await?;
    let watcher = subxt::TransactionProgress::new(sub, client, ext_hash);

    // Create a non-empty block
    let _: Value = client
        .rpc()
        .client
        .request(
            "engine_createBlock",
            &[Value::Bool(false), Value::Bool(false), Value::Null],
        )
        .await?;

    // Get extrinsic events
    watcher
        .wait_for_in_block()
        .await?
        .fetch_events()
        .await
        .map_err(Into::into)
}
