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
use common::*;
use cucumber::{given, then, when, World, WorldInit};
use sp_keyring::AccountKeyring;
use std::convert::Infallible;
use std::str::FromStr;

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

fn parse_amount(amount: u64, unit: &str) -> (u64, bool) {
    match unit {
        "ĞD" => (amount * 100, false),
        "cĞD" => (amount, false),
        "UD" => (amount * 1_000, true),
        "mUD" => (amount, true),
        _ => unreachable!(),
    }
}

#[given(regex = r"([a-zA-Z]+) have (\d+) (ĞD|cĞD|UD|mUD)")]
async fn who_have(world: &mut DuniterWorld, who: String, amount: u64, unit: String) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who).expect("unknown to");
    let (mut amount, is_ud) = parse_amount(amount, &unit);

    if is_ud {
        let current_ud_amount = world
            .api
            .storage()
            .universal_dividend()
            .current_ud_storage(None)
            .await?;
        amount = (amount * current_ud_amount) / 1_000;
    }

    // Create {amount} ĞD for {who}
    common::balances::set_balance(&world.api, &world.client, who, amount).await?;

    Ok(())
}

#[when(regex = r"([a-zA-Z]+) send (\d+) (ĞD|cĞD|UD|mUD) to ([a-zA-Z]+)")]
async fn transfer(
    world: &mut DuniterWorld,
    from: String,
    amount: u64,
    unit: String,
    to: String,
) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");
    let (amount, is_ud) = parse_amount(amount, &unit);

    if is_ud {
        common::balances::transfer_ud(&world.api, &world.client, from, amount, to).await
    } else {
        common::balances::transfer(&world.api, &world.client, from, amount, to).await
    }
}

#[when(regex = r"([a-zA-Z]+) sends all (?:his|her) (?:ĞDs?|DUs?) to ([a-zA-Z]+)")]
async fn send_all_to(world: &mut DuniterWorld, from: String, to: String) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");

    common::balances::transfer_all(&world.api, &world.client, from, to).await
}

#[then(regex = r"([a-zA-Z]+) should have (\d+) ĞD")]
async fn should_have(world: &mut DuniterWorld, who: String, amount: u64) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who)
        .expect("unknown to")
        .to_account_id();
    let amount = amount * 100;

    let who_account = world.api.storage().system().account(who, None).await?;
    assert_eq!(who_account.data.free, amount);
    Ok(())
}

#[then(regex = r"current UD amount should be (\d+).(\d+)")]
async fn current_ud_amount_should_be(
    world: &mut DuniterWorld,
    amount: u64,
    cents: u64,
) -> Result<()> {
    // Parse inputs
    let expected_amount = amount + (cents * 100);

    let current_ud_amount = world
        .api
        .storage()
        .universal_dividend()
        .current_ud_storage(None)
        .await?;
    assert_eq!(current_ud_amount, expected_amount);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    //env_logger::init();

    DuniterWorld::cucumber()
		.fail_on_skipped()
		.max_concurrent_scenarios(4)
		.run("features").await;
}
