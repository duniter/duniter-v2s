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

mod common;

use async_trait::async_trait;
use common::node_runtime::runtime_types::gdev_runtime;
use common::node_runtime::runtime_types::pallet_balances;
use common::*;
use cucumber::{given, then, when, World, WorldInit};
use sp_keyring::AccountKeyring;
use std::convert::Infallible;
use std::str::FromStr;
use subxt::{sp_runtime::MultiAddress, PairSigner};

#[derive(WorldInit)]
pub struct DuniterWorld {
    api: Api,
    client: Client,
    _process: Process,
}

impl std::fmt::Debug for DuniterWorld {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[async_trait(?Send)]
impl World for DuniterWorld {
    // We do require some error type.
    type Error = Infallible;

    async fn new() -> std::result::Result<Self, Infallible> {
        let (api, client, _process) = spawn_node().await;
        Ok(DuniterWorld {
            api,
            client,
            _process,
        })
    }
}

#[given(regex = r"([a-zA-Z]+) have (\d+) ĞD")]
async fn who_have(world: &mut DuniterWorld, who: String, amount: u64) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who)
        .expect("unknown to")
        .to_account_id();
    let amount = amount * 100;

    // Create {amount} ĞD for {who}
    let _events = create_block_with_extrinsic(
        &world.client,
        world
            .api
            .tx()
            .sudo()
            .sudo(gdev_runtime::Call::Balances(
                pallet_balances::pallet::Call::set_balance {
                    who: MultiAddress::Id(who),
                    new_free: amount,
                    new_reserved: 0,
                },
            ))
            .create_signed(&PairSigner::new(SUDO_ACCOUNT.pair()), ())
            .await?,
    )
    .await?;

    Ok(())
}

#[when(regex = r"([a-zA-Z]+) send (\d+) ĞD to ([a-zA-Z]+)")]
async fn transfer(world: &mut DuniterWorld, from: String, amount: u64, to: String) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let amount = amount * 100;
    let to = AccountKeyring::from_str(&to).expect("unknown to");

    common::balances::transfer(&world.api, &world.client, from, amount, to).await
}

#[when(regex = r"([a-zA-Z]+) sends all (?:his|her) ĞDs? to ([a-zA-Z]+)")]
async fn send_all_to(world: &mut DuniterWorld, from: String, to: String) -> Result<()> {
    // Parse inputs
    let from = PairSigner::new(
        AccountKeyring::from_str(&from)
            .expect("unknown from")
            .pair(),
    );
    let to = AccountKeyring::from_str(&to)
        .expect("unknown to")
        .to_account_id();

    let _events = create_block_with_extrinsic(
        &world.client,
        world
            .api
            .tx()
            .balances()
            .transfer_all(to.clone().into(), false)
            .create_signed(&from, ())
            .await?,
    )
    .await?;

    Ok(())
}

#[then(regex = r"([a-zA-Z]+) have (\d+) ĞD")]
async fn assert_who_have(world: &mut DuniterWorld, who: String, amount: u64) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who)
        .expect("unknown to")
        .to_account_id();
    let amount = amount * 100;

    let who_account = world.api.storage().system().account(who, None).await?;
    assert_eq!(who_account.data.free, amount);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    //env_logger::init();

    DuniterWorld::run("tests/features").await
}
