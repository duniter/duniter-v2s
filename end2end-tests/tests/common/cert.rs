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
use super::gdev::runtime_types::pallet_certification;
use super::*;
use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, utils::MultiAddress};

pub async fn certify(client: &FullClient, from: AccountKeyring, to: AccountKeyring) -> Result<()> {
    let signer = PairSigner::new(from.pair());
    let from = from.to_account_id();
    let to = to.to_account_id();

    let _issuer_index = client
        .client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .fetch(
            &gdev::storage()
                .identity()
                .identity_index_of(&from.clone().into()),
        )
        .await?
        .unwrap_or_else(|| panic!("{} issuer must exist", from));
    let receiver_index = client
        .client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .fetch(&gdev::storage().identity().identity_index_of(&to.into()))
        .await?
        .unwrap_or_else(|| panic!("{} issuer must exist", from));

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().certification().add_cert(receiver_index),
                &signer,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}
