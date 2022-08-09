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

use super::node_runtime::runtime_types::gdev_runtime;
use super::node_runtime::runtime_types::pallet_balances;
use super::node_runtime::runtime_types::pallet_oneshot_account;
use super::*;
use sp_keyring::AccountKeyring;
use subxt::{
    sp_runtime::{AccountId32, MultiAddress},
    PairSigner,
};

pub enum Account {
    Normal(AccountKeyring),
    Oneshot(AccountKeyring),
}

impl Account {
    fn to_account_id(
        &self,
    ) -> pallet_oneshot_account::types::Account<MultiAddress<AccountId32, ()>> {
        match self {
            Account::Normal(account) => {
                pallet_oneshot_account::types::Account::Normal(account.to_account_id().into())
            }
            Account::Oneshot(account) => {
                pallet_oneshot_account::types::Account::Oneshot(account.to_account_id().into())
            }
        }
    }
}

pub async fn create_oneshot_account(
    api: &Api,
    client: &Client,
    from: AccountKeyring,
    amount: u64,
    to: AccountKeyring,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        api.tx()
            .oneshot_account()
            .create_oneshot_account(to.into(), amount)?
            .create_signed(&from, BaseExtrinsicParamsBuilder::new())
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn consume_oneshot_account(
    api: &Api,
    client: &Client,
    from: AccountKeyring,
    to: Account,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        api.tx()
            .oneshot_account()
            .consume_oneshot_account(0, to)?
            .create_signed(&from, BaseExtrinsicParamsBuilder::new())
            .await?,
    )
    .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn consume_oneshot_account_with_remaining(
    api: &Api,
    client: &Client,
    from: AccountKeyring,
    amount: u64,
    to: Account,
    remaining_to: Account,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();
    let remaining_to = remaining_to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        api.tx()
            .oneshot_account()
            .consume_oneshot_account_with_remaining(0, to, remaining_to, amount)?
            .create_signed(&from, BaseExtrinsicParamsBuilder::new())
            .await?,
    )
    .await?;

    Ok(())
}
