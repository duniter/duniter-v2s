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

use super::{gdev, gdev::runtime_types::pallet_balances, *};
use crate::common::pair_signer::PairSigner;
use sp_keyring::sr25519::Keyring;
use subxt::utils::MultiAddress;

pub async fn set_balance(client: &FullClient, who: Keyring, amount: u64) -> Result<()> {
    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx()
                    .sudo()
                    .sudo(gdev::runtime_types::gdev_runtime::RuntimeCall::Balances(
                        pallet_balances::pallet::Call::force_set_balance {
                            who: MultiAddress::Id(who.to_raw_public().into()),
                            new_free: amount,
                        },
                    )),
                &PairSigner::new(SUDO_ACCOUNT.pair()),
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn transfer(client: &FullClient, from: Keyring, amount: u64, to: Keyring) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = MultiAddress::Id(to.to_raw_public().into());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().universal_dividend().transfer_ud(to, amount),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn transfer_all(client: &FullClient, from: Keyring, to: Keyring) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = MultiAddress::Id(to.to_raw_public().into());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().balances().transfer_all(to, false),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn transfer_ud(
    client: &FullClient,
    from: Keyring,
    amount: u64,
    to: Keyring,
) -> Result<()> {
    let from = PairSigner::new(from.pair());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx()
                    .universal_dividend()
                    .transfer_ud(MultiAddress::Id(to.to_raw_public().into()), amount),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}
