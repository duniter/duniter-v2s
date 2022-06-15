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
use super::node_runtime::runtime_types::pallet_certification;
use super::*;
use sp_keyring::AccountKeyring;
use subxt::{sp_runtime::MultiAddress, PairSigner};

pub async fn certify(
    api: &Api,
    client: &Client,
    from: AccountKeyring,
    to: AccountKeyring,
) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_account_id();

    let _events = create_block_with_extrinsic(
        client,
        api.tx()
            .cert()
            .add_cert(to)
            .create_signed(&from, ())
            .await?,
    )
    .await?;

    Ok(())
}
