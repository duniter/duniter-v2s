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

use super::*;
use sp_keyring::AccountKeyring;
use subxt::tx::PairSigner;

pub async fn claim_membership(client: &Client, from: AccountKeyring) -> Result<()> {
    let from = PairSigner::new(from.pair());

    let _events = create_block_with_extrinsic(
        client,
        client
            .tx()
            .create_signed(
                &gdev::tx().membership().claim_membership(),
                &from,
                BaseExtrinsicParamsBuilder::new(),
            )
            .await
            .unwrap(),
    )
    .await
    .unwrap();

    Ok(())
}
