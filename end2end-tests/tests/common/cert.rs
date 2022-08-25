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

use super::gdev;
use super::gdev::runtime_types::pallet_certification;
use super::*;
use sp_keyring::AccountKeyring;
use subxt::{ext::sp_runtime::MultiAddress, tx::PairSigner};

pub async fn certify(client: &Client, from: AccountKeyring, to: AccountKeyring) -> Result<()> {
    let signer = PairSigner::new(from.pair());
    let from = from.to_account_id();
    let to = to.to_account_id();

    let issuer_index = client
        .storage()
        .fetch(&gdev::storage().identity().identity_index_of(&from), None)
        .await?
        .unwrap();
    let receiver_index = client
        .storage()
        .fetch(&gdev::storage().identity().identity_index_of(&to), None)
        .await?
        .unwrap();

    let _events = create_block_with_extrinsic(
        client,
        client
            .tx()
            .create_signed(
                &gdev::tx().cert().add_cert(issuer_index, receiver_index),
                &signer,
                BaseExtrinsicParamsBuilder::new(),
            )
            .await?,
    )
    .await?;

    Ok(())
}
