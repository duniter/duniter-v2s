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

use super::{
    gdev,
    gdev::runtime_types::{pallet_balances, pallet_oneshot_account},
    *,
};
use crate::common::pair_signer::PairSigner;
use sp_keyring::sr25519::Keyring;
use subxt::utils::{AccountId32, MultiAddress};

pub enum Account {
    Normal(Keyring),
    Oneshot(Keyring),
}

impl Account {
    fn to_account_id(
        &self,
    ) -> pallet_oneshot_account::types::Account<MultiAddress<AccountId32, ()>> {
        match self {
            Account::Normal(account) => pallet_oneshot_account::types::Account::Normal(
                MultiAddress::Id(account.to_raw_public().into()),
            ),
            Account::Oneshot(account) => pallet_oneshot_account::types::Account::Oneshot(
                MultiAddress::Id(account.to_raw_public().into()),
            ),
        }
    }
}

pub async fn create_oneshot_account(
    client: &FullClient,
    from: Keyring,
    amount: u64,
    to: Keyring,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = MultiAddress::Id(to.to_raw_public().into());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx()
                    .oneshot_account()
                    .create_oneshot_account(to, amount),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn consume_oneshot_account(
    client: &FullClient,
    from: Keyring,
    to: Account,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().oneshot_account().consume_oneshot_account(0, to),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn consume_oneshot_account_with_remaining(
    client: &FullClient,
    from: Keyring,
    amount: u64,
    to: Account,
    remaining_to: Account,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();
    let remaining_to = remaining_to.to_account_id();

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx()
                    .oneshot_account()
                    .consume_oneshot_account_with_remaining(0, to, remaining_to, amount),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}
