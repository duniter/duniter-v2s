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
pub mod setup;

#[subxt::subxt(
    runtime_metadata_path = "../resources/gtest_metadata.scale",
    derive_for_all_types = "Eq, PartialEq, codec::Decode"
)]
pub mod gtest {}

use anyhow::anyhow;
use codec::Encode;
use serde_json::Value;
use sp_keyring::sr25519::Keyring;
use std::{
    path::PathBuf,
    process::Command,
    str::FromStr,
    time::{Duration, Instant},
};
use subxt::{
    backend::rpc::RpcClient,
    config::{SubstrateExtrinsicParams, substrate::SubstrateExtrinsicParamsBuilder},
    ext::subxt_rpcs::client::{RpcParams, rpc_params},
};

pub type Client = subxt::OnlineClient<GtestConfig>;
pub type Event = gtest::Event;
pub type Result<T> = anyhow::Result<T>;
pub type SubmittableExtrinsic = subxt::tx::SubmittableTransaction<GtestConfig, Client>;
pub type TxProgress = subxt::tx::TxProgress<GtestConfig, Client>;

pub enum GtestConfig {}
impl subxt::config::Config for GtestConfig {
    type AccountId = subxt::utils::AccountId32;
    type Address = subxt::utils::MultiAddress<Self::AccountId, u32>;
    type AssetId = ();
    type ExtrinsicParams = SubstrateExtrinsicParams<Self>;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type Header =
        subxt::config::substrate::SubstrateHeader<u32, subxt::config::substrate::BlakeTwo256>;
    type Signature = subxt::utils::MultiSignature;
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

pub const SUDO_ACCOUNT: sp_keyring::ed25519::Keyring = sp_keyring::ed25519::Keyring::Alice;

pub struct Process(std::process::Child);
impl Process {
    pub fn kill(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            self.0.kill().expect("failed to stop node");
            self.0.wait().expect("failed to reap node");
        }
    }
}

// Do not let the process keep running after the tests ended
impl Drop for Process {
    fn drop(&mut self) {
        self.kill()
    }
}

pub const DISTANCE_ORACLE_LOCAL_PATH: &str = "../target/debug/distance-oracle";
const DUNITER_DOCKER_PATH: &str = "/usr/local/bin/duniter-gtest-fast";
const DUNITER_LOCAL_PATH: &str = "../target/debug/duniter";

struct FullNode {
    process: Process,
    p2p_port: u16,
    rpc_port: u16,
}

pub async fn spawn_node(no_spawn: bool) -> (FullClient, Option<Process>, u16) {
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
                "--chain=gt_local",
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
        );
        opt_process = Some(process);
        the_rpc_port = rpc_port;
    }
    let timeout = std::env::var("DUNITER_END2END_TESTS_SPAWN_NODE_TIMEOUT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(30);
    let deadline = Instant::now() + Duration::from_secs(timeout);
    let (rpc, client) = loop {
        if let Ok(rpc) = RpcClient::from_url(format!("ws://127.0.0.1:{the_rpc_port}")).await
            && let Ok(client) = Client::from_rpc_client(rpc.clone()).await
        {
            break (rpc, client);
        }
        if let Some(process) = &mut opt_process
            && let Some(status) = process.0.try_wait().expect("cannot inspect node")
        {
            panic!("node exited with {status}; read duniter-v2s-{the_rpc_port}.log");
        }
        assert!(
            Instant::now() < deadline,
            "RPC startup timeout; read duniter-v2s-{the_rpc_port}.log"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    };

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
) -> Result<subxt::blocks::ExtrinsicEvents<GtestConfig>> {
    //println!("extrinsic encoded: {}", hex::encode(extrinsic.encoded()));

    let mut watcher = extrinsic.submit_and_watch().await?;
    loop {
        let status = tokio::time::timeout(Duration::from_secs(30), watcher.next())
            .await?
            .ok_or_else(|| anyhow!("transaction subscription ended before validation"))??;
        match status {
            subxt::tx::TxStatus::Validated | subxt::tx::TxStatus::Broadcasted => break,
            subxt::tx::TxStatus::Invalid { message }
            | subxt::tx::TxStatus::Dropped { message }
            | subxt::tx::TxStatus::Error { message } => {
                return Err(anyhow!("transaction rejected: {message}"));
            }
            _ => {}
        }
    }

    // Epoch-transition hooks can consume the available block budget. Allow
    // inclusion in a subsequent block, and always verify the finalized result.
    let mut finalized = None;
    for attempt in 0..4 {
        let _: Value = client
            .request("engine_createBlock", rpc_params![true, true, Value::Null])
            .await?;
        // The subscription can lag block import on a busy CI runner. After the
        // last block, allow notification delivery without sealing extra blocks.
        let delivery_timeout = if attempt == 3 {
            Duration::from_secs(30)
        } else {
            Duration::from_millis(100)
        };
        let deadline = tokio::time::Instant::now() + delivery_timeout;
        loop {
            let Ok(status) = tokio::time::timeout_at(deadline, watcher.next()).await else {
                break;
            };
            match status
                .ok_or_else(|| anyhow!("transaction subscription ended before finalization"))??
            {
                subxt::tx::TxStatus::InFinalizedBlock(block) => {
                    finalized = Some(block);
                    break;
                }
                subxt::tx::TxStatus::Invalid { message }
                | subxt::tx::TxStatus::Dropped { message }
                | subxt::tx::TxStatus::Error { message } => {
                    return Err(anyhow!("transaction rejected: {message}"));
                }
                _ => {}
            }
        }
        if finalized.is_some() {
            break;
        }
    }
    let events = finalized
        .ok_or_else(|| anyhow!("transaction was not finalized within four manually sealed blocks"))?
        .wait_for_success()
        .await?;
    for result in events.find::<gtest::sudo::events::Sudid>() {
        let result = result?;
        if let Err(error) = result.sudo_result {
            return Err(anyhow!("sudo call failed: {error:?}"));
        }
    }
    Ok(events)
}

fn spawn_full_node(args: &[&str], duniter_binary_path: &str) -> FullNode {
    // Ports
    let p2p_port = portpicker::pick_unused_port().expect("No ports free");
    let rpc_port = portpicker::pick_unused_port().expect("No ports free");

    // Logs
    let log_file_path = format!("duniter-v2s-{rpc_port}.log");
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
            .env("DUNITER_SUDO_ACCOUNT_CRYPTO", "ed25519")
            .stdout(std::process::Stdio::null())
            .stderr(log_file)
            .spawn()
            .expect("failed to spawn node"),
    );

    FullNode {
        process,
        p2p_port,
        rpc_port,
    }
}

pub fn spawn_distance_oracle(distance_oracle_binary_path: &str, duniter_rpc_port: u16) {
    Command::new(distance_oracle_binary_path)
        .args(
            [
                "-u",
                &format!("ws://127.0.0.1:{duniter_rpc_port}"),
                "-d",
                "/tmp/duniter-cucumber/chains/gtest/distance",
            ]
            .iter(),
        )
        .spawn()
        .expect("failed to spawn distance oracle")
        .wait()
        .unwrap();
}

/// Sign sr25519 scenario calls and ed25519 sudo calls with the same test configuration.
mod pair_signer {
    use super::*;
    use sp_core::{Pair, sr25519};
    use sp_runtime::{MultiSigner, traits::IdentifyAccount};
    use subxt::{
        Config,
        config::substrate::{AccountId32, MultiSignature},
        tx::Signer,
    };

    #[derive(Clone)]
    pub struct PairSigner<P: Pair = sr25519::Pair> {
        account_id: <GtestConfig as Config>::AccountId,
        signer: P,
    }

    impl<P: Pair> PairSigner<P>
    where
        MultiSigner: From<P::Public>,
    {
        pub fn new(signer: P) -> Self {
            let account_id = MultiSigner::from(signer.public()).into_account();
            Self {
                account_id: AccountId32(account_id.into()),
                signer,
            }
        }

        pub fn signer(&self) -> &P {
            &self.signer
        }

        pub fn account_id(&self) -> &AccountId32 {
            &self.account_id
        }
    }

    impl Signer<GtestConfig> for PairSigner<sr25519::Pair> {
        fn account_id(&self) -> <GtestConfig as Config>::AccountId {
            self.account_id.clone()
        }

        fn sign(&self, signer_payload: &[u8]) -> <GtestConfig as Config>::Signature {
            MultiSignature::Sr25519(self.signer.sign(signer_payload).0)
        }
    }

    impl Signer<GtestConfig> for PairSigner<sp_core::ed25519::Pair> {
        fn account_id(&self) -> <GtestConfig as Config>::AccountId {
            self.account_id.clone()
        }

        fn sign(&self, signer_payload: &[u8]) -> <GtestConfig as Config>::Signature {
            MultiSignature::Ed25519(self.signer.sign(signer_payload).0)
        }
    }
}
