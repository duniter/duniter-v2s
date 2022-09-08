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

use super::gdev;
use super::gdev::runtime_types::pallet_balances;
use super::*;
use sp_keyring::AccountKeyring;
use subxt::{ext::sp_runtime::MultiAddress, tx::PairSigner};

pub async fn set_balance(client: &Client, who: AccountKeyring, amount: u64) -> Result<()> {
    let _events = create_block_with_extrinsic(
        client,
        client
            .tx()
            .create_signed(
                &gdev::tx()
                    .sudo()
                    .sudo(gdev::runtime_types::gdev_runtime::Call::Balances(
                        pallet_balances::pallet::Call::set_balance {
                            who: MultiAddress::Id(who.to_account_id()),
                            new_free: amount,
                            new_reserved: 0,
                        },
                    )),
                &PairSigner::new(SUDO_ACCOUNT.pair()),
                BaseExtrinsicParamsBuilder::new(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn transfer(
    client: &Client,
    from: AccountKeyring,
    amount: u64,
    to: AccountKeyring,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        client
            .tx()
            .create_signed(
                &gdev::tx()
                    .universal_dividend()
                    .transfer_ud(to.clone().into(), amount),
                &from,
                BaseExtrinsicParamsBuilder::new(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn transfer_all(client: &Client, from: AccountKeyring, to: AccountKeyring) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        client
            .tx()
            .create_signed(
                &gdev::tx().balances().transfer_all(to.clone().into(), false),
                &from,
                BaseExtrinsicParamsBuilder::new(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn transfer_ud(
    client: &Client,
    from: AccountKeyring,
    amount: u64,
    to: AccountKeyring,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        client
            .tx()
            .create_signed(
                &gdev::tx()
                    .universal_dividend()
                    .transfer_ud(to.clone().into(), amount),
                &from,
                BaseExtrinsicParamsBuilder::new(),
            )
            .await?,
    )
    .await?;

    Ok(())
}
