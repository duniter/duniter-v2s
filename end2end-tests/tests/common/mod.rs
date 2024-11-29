// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::enum_variant_names, dead_code, unused_imports)]

pub mod balances;
pub mod cert;
pub mod distance;
pub mod identity;
pub mod oneshot;

#[subxt::subxt(
    runtime_metadata_path = "../resources/metadata.scale",
    derive_for_all_types = "Eq, PartialEq"
)]
pub mod gdev {}

use anyhow::anyhow;
use codec::Encode;
use notify_debouncer_mini::new_debouncer;
use serde_json::Value;
use sp_keyring::AccountKeyring;
use std::{
    io::prelude::*,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    time::{Duration, Instant},
};
use subxt::{
    backend::rpc::{RpcClient, RpcParams},
    config::{substrate::SubstrateExtrinsicParamsBuilder, SubstrateExtrinsicParams},
    ext::{sp_core, sp_runtime},
    rpc_params,
};

pub type Client = subxt::OnlineClient<GdevConfig>;
pub type Event = gdev::Event;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type SubmittableExtrinsic = subxt::tx::SubmittableExtrinsic<GdevConfig, Client>;
pub type TxProgress = subxt::tx::TxProgress<GdevConfig, Client>;

pub enum GdevConfig {}
impl subxt::config::Config for GdevConfig {
    type AccountId = subxt::utils::AccountId32;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type AssetId = ();
    type ExtrinsicParams = SubstrateExtrinsicParams<Self>;
    type Hash = subxt::utils::H256;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type Header =
        subxt::config::substrate::SubstrateHeader<u32, subxt::config::substrate::BlakeTwo256>;
    type Signature = sp_runtime::MultiSignature;
}

#[derive(Copy, Clone, Debug, Default, Encode)]
pub struct Tip {
    #[codec(compact)]
    tip: u64,
}

pub struct FullClient {
    pub rpc: RpcClient,
    pub client: Client,
}

impl Tip {
    pub fn new(amount: u64) -> Self {
        Tip { tip: amount }
    }
}

impl From<u64> for Tip {
    fn from(n: u64) -> Self {
        Self::new(n)
    }
}

pub const SUDO_ACCOUNT: AccountKeyring = AccountKeyring::Alice;

pub struct Process(std::process::Child);
impl Process {
    pub fn kill(&mut self) {
        self.0.kill().expect("node already down");
    }
}

// Do not let the process keep running after the tests ended
impl Drop for Process {
    fn drop(&mut self) {
        self.kill()
    }
}

pub const DISTANCE_ORACLE_LOCAL_PATH: &str = "../target/debug/distance-oracle";
const DUNITER_DOCKER_PATH: &str = "/usr/local/bin/duniter";
const DUNITER_LOCAL_PATH: &str = "../target/debug/duniter";

struct FullNode {
    process: Process,
    p2p_port: u16,
    rpc_port: u16,
}

pub async fn spawn_node(
    maybe_genesis_conf_file: Option<PathBuf>,
    no_spawn: bool,
) -> (FullClient, Option<Process>, u16) {
    println!("maybe_genesis_conf_file={:?}", maybe_genesis_conf_file);
    let duniter_binary_path = std::env::var("DUNITER_BINARY_PATH").unwrap_or_else(|_| {
        if std::path::Path::new(DUNITER_DOCKER_PATH).exists() {
            DUNITER_DOCKER_PATH.to_owned()
        } else {
            DUNITER_LOCAL_PATH.to_owned()
        }
    });

    let mut the_rpc_port = 9944;
    let mut opt_process = None;
    // Eventually spawn a node (we most likely will - unless --no-spawn option is used)
    if !no_spawn {
        let FullNode {
            process,
            p2p_port: _,
            rpc_port,
        } = spawn_full_node(
            &[
                "--chain=gdev_dev",
                "--execution=Native",
                "--sealing=manual",
                // Necessary options which were previously set by --dev option:
                "--force-authoring",
                "--rpc-cors=all",
                "--alice",
                "--tmp",
                "--unsafe-force-node-key-generation",
                // Fix: End2End test may fail due to network discovery. This option disables automatic peer discovery.π
                "--reserved-only",
                // prevent local network discovery (even it does not connect due to above flag)
                "--no-mdns",
            ],
            &duniter_binary_path,
            maybe_genesis_conf_file,
        );
        opt_process = Some(process);
        the_rpc_port = rpc_port;
    }
    let rpc = RpcClient::from_url(format!("ws://127.0.0.1:{}", the_rpc_port))
        .await
        .expect("Failed to create the rpc backend");
    let client = Client::from_rpc_client(rpc.clone()).await.unwrap();

    (FullClient { rpc, client }, opt_process, the_rpc_port)
}

pub async fn create_empty_block(client: &RpcClient) -> Result<()> {
    // Create an empty block
    let _: Value = client
        .request("engine_createBlock", rpc_params![true, true, Value::Null])
        .await?;

    Ok(())
}

pub async fn create_block_with_extrinsic(
    client: &RpcClient,
    extrinsic: SubmittableExtrinsic,
) -> Result<subxt::blocks::ExtrinsicEvents<GdevConfig>> {
    //println!("extrinsic encoded: {}", hex::encode(extrinsic.encoded()));

    let watcher = extrinsic.submit_and_watch().await?;

    // Create a non-empty block
    let _: Value = client
        .request("engine_createBlock", rpc_params![false, true, Value::Null])
        .await?;

    // Get extrinsic events
    watcher
        .wait_for_finalized()
        .await?
        .fetch_events()
        .await
        .map_err(Into::into)
}

fn spawn_full_node(
    args: &[&str],
    duniter_binary_path: &str,
    maybe_genesis_conf_file: Option<PathBuf>,
) -> FullNode {
    // Ports
    let p2p_port = portpicker::pick_unused_port().expect("No ports free");
    let rpc_port = portpicker::pick_unused_port().expect("No ports free");

    // Env vars
    let mut envs = Vec::new();
    if let Some(genesis_conf_file) = maybe_genesis_conf_file {
        envs.push(("DUNITER_GENESIS_CONFIG", genesis_conf_file.clone()));
        envs.push(("DUNITER_GENESIS_DATA", genesis_conf_file));
    }

    // Logs
    let log_file_path = format!("duniter-v2s-{}.log", rpc_port);
    let log_file = std::fs::File::create(&log_file_path).expect("fail to create log file");

    // Command
    let process = Process(
        Command::new(duniter_binary_path)
            .args(
                [
                    "--no-telemetry",
                    "--no-prometheus",
                    "--port",
                    &p2p_port.to_string(),
                    "--rpc-port",
                    &rpc_port.to_string(),
                ]
                .iter()
                .chain(args),
            )
            .envs(envs)
            .stdout(std::process::Stdio::null())
            .stderr(log_file)
            .spawn()
            .expect("failed to spawn node"),
    );

    let timeout =
        if let Ok(duration_string) = std::env::var("DUNITER_END2END_TESTS_SPAWN_NODE_TIMEOUT") {
            duration_string.parse().unwrap_or(10)
        } else {
            10
        };

    wait_until_log_line(
        "***** Duniter has fully started *****",
        &log_file_path,
        std::time::Duration::from_secs(timeout),
    );

    FullNode {
        process,
        p2p_port,
        rpc_port,
    }
}

fn wait_until_log_line(expected_log_line: &str, log_file_path: &str, timeout: Duration) {
    if cfg!(target_os = "macos") {
        // MacOs seems to not be able to use inotify (buggy)
        // So we use a specific implementation for `wait_until_log_line()` here
        let start = Instant::now();
        loop {
            let now = Instant::now();
            if now.duration_since(start) > timeout {
                eprintln!("Timeout starting node");
                std::process::exit(1);
            }
            if has_log_line(log_file_path, expected_log_line) {
                // Ready
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    } else {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(std::time::Duration::from_millis(100), tx).unwrap();
        debouncer
            .watcher()
            .watch(
                Path::new(log_file_path),
                notify::RecursiveMode::NonRecursive,
            )
            .unwrap();

        let mut pos = 0;
        loop {
            match rx.recv_timeout(timeout) {
                Ok(_) => {
                    let mut file = std::fs::File::open(log_file_path).unwrap();
                    file.seek(std::io::SeekFrom::Start(pos)).unwrap();
                    pos = file.metadata().unwrap().len();
                    let reader = std::io::BufReader::new(file);

                    for line in reader.lines() {
                        if line.expect("fail to read line").contains(expected_log_line) {
                            return;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn has_log_line(log_file_path: &str, expected_log_line: &str) -> bool {
    let mut file = std::fs::File::open(log_file_path).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        if line.expect("fail to read line").contains(expected_log_line) {
            return true;
        }
    }
    false
}

pub fn spawn_distance_oracle(distance_oracle_binary_path: &str, duniter_rpc_port: u16) {
    Command::new(distance_oracle_binary_path)
        .args(
            [
                "-u",
                &format!("ws://127.0.0.1:{duniter_rpc_port}"),
                "-d",
                "/tmp/duniter-cucumber/chains/gdev/distance",
            ]
            .iter(),
        )
        .spawn()
        .expect("failed to spawn distance oracle")
        .wait()
        .unwrap();
}
