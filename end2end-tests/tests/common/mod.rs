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

#![allow(clippy::enum_variant_names, dead_code, unused_imports)]

pub mod balances;

#[subxt::subxt(runtime_metadata_path = "../resources/metadata.scale")]
pub mod node_runtime {}

use serde_json::Value;
use sp_keyring::AccountKeyring;
use std::io::prelude::*;
use std::process::Command;
use std::str::FromStr;
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

struct FullNode {
    process: Process,
    p2p_port: u16,
    ws_port: u16,
}

pub async fn spawn_node() -> (Api, Client, Process) {
    let duniter_binary_path = std::env::var("DUNITER_BINARY_PATH")
        .unwrap_or_else(|_| "../target/debug/duniter".to_owned());
    let FullNode {
        process,
        p2p_port: _,
        ws_port,
    } = spawn_full_node(
        &duniter_binary_path,
        &["--dev", "--execution=Native", "--sealing=manual"],
    );
    let client = ClientBuilder::new()
        .set_url(format!("ws://127.0.0.1:{}", ws_port))
        .build()
        .await
        .expect("fail to connect to node");
    let api = client.clone().to_runtime_api::<Api>();

    (api, client, process)
}

pub async fn create_empty_block(client: &Client) -> Result<()> {
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
}

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

fn spawn_full_node(duniter_binary_path: &str, args: &[&str]) -> FullNode {
    let p2p_port = portpicker::pick_unused_port().expect("No ports free");
    let rpc_port = portpicker::pick_unused_port().expect("No ports free");
    let ws_port = portpicker::pick_unused_port().expect("No ports free");
    let log_file_path = format!("duniter-v2s-{}.log", ws_port);
    let log_file = std::fs::File::create(&log_file_path).expect("fail to create log file");
    let process = Process(
        Command::new(duniter_binary_path)
            .args(
                [
                    "--no-telemetry",
                    "--no-prometheus",
                    "--tmp",
                    "--port",
                    &p2p_port.to_string(),
                    "--rpc-port",
                    &rpc_port.to_string(),
                    "--ws-port",
                    &ws_port.to_string(),
                ]
                .iter()
                .chain(args),
            )
            .stdout(std::process::Stdio::null())
            .stderr(log_file)
            .spawn()
            .expect("failed to spawn node"),
    );

    let timeout =
        if let Ok(duration_string) = std::env::var("DUNITER_END2END_TESTS_SPAWN_NODE_TIMEOUT") {
            duration_string.parse().unwrap_or(4)
        } else {
            4
        };

    wait_until_log_line(
        "***** Duniter has fully started *****",
        &log_file_path,
        std::time::Duration::from_secs(timeout),
    );

    FullNode {
        process,
        p2p_port,
        ws_port,
    }
}

fn wait_until_log_line(expected_log_line: &str, log_file_path: &str, timeout: std::time::Duration) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, std::time::Duration::from_millis(100)).unwrap();
    use notify::Watcher as _;
    watcher
        .watch(&log_file_path, notify::RecursiveMode::NonRecursive)
        .unwrap();

    let mut pos = 0;
    loop {
        match rx.recv_timeout(timeout) {
            Ok(notify::DebouncedEvent::Write(_)) => {
                let mut file = std::fs::File::open(&log_file_path).unwrap();
                file.seek(std::io::SeekFrom::Start(pos)).unwrap();
                pos = file.metadata().unwrap().len();
                let reader = std::io::BufReader::new(file);

                for line in reader.lines() {
                    if line.expect("fail to read line").contains(expected_log_line) {
                        return;
                    }
                }
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}
