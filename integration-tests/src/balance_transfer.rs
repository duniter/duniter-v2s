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

use crate::common::*;
use sp_keyring::AccountKeyring;
use subxt::PairSigner;

#[tokio::test]
async fn test_balance_transfer() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Spawn a node
    let (api, client, _process) = spawn_node().await;

    let alice = PairSigner::new(AccountKeyring::Alice.pair());
    let dave = AccountKeyring::Dave.to_account_id();

    let events = create_block_with_extrinsic(
        &client,
        api.tx()
            .balances()
            .transfer(dave.clone().into(), 512)
            .create_signed(&alice, ())
            .await?,
    )
    .await?;

    println!(
        "Balance transfer extrinsic written in blockchain, events: {:?}",
        events
    );

    // verify that Bob's free Balance increased
    let dave_post = api.storage().system().account(dave, None).await?;
    println!("Bob's Free Balance is now {}\n", dave_post.data.free);
    assert_eq!(dave_post.data.free, 512);

    Ok(())
}
