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

use super::{gdev, gdev::runtime_types::pallet_identity, *};
use crate::{
    common::pair_signer::PairSigner, gdev::runtime_types::pallet_identity::types::IdtyName,
    DuniterWorld,
};
use sp_keyring::sr25519::Keyring;
use subxt::config::substrate::MultiAddress;

type BlockNumber = u32;
type AccountId = subxt::utils::AccountId32;
type IdtyData = gdev::runtime_types::common_runtime::entities::IdtyData;
type IdtyValue =
    gdev::runtime_types::pallet_identity::types::IdtyValue<BlockNumber, AccountId, IdtyData>;

// submit extrinsics

pub async fn create_identity(client: &FullClient, from: Keyring, to: Keyring) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let to = to.to_raw_public();

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().identity().create_identity(to.into()),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn confirm_identity(client: &FullClient, from: Keyring, pseudo: String) -> Result<()> {
    let from = PairSigner::new(from.pair());
    let pseudo: IdtyName = IdtyName(pseudo.as_bytes().to_vec());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().identity().confirm_identity(pseudo),
                &from,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

// get identity index from account keyring name
pub async fn get_identity_index(world: &DuniterWorld, account: String) -> Result<u32> {
    let account: AccountId = Keyring::from_str(&account)
        .expect("unknown account")
        .to_raw_public()
        .into();

    let identity_index = world
        .read(&gdev::storage().identity().identity_index_of(&account))
        .await
        .await?
        .ok_or_else(|| anyhow::anyhow!("identity {} has no associated index", account))
        .unwrap();

    Ok(identity_index)
}
// get identity value from account keyring name
pub async fn get_identity_value(world: &DuniterWorld, account: String) -> Result<IdtyValue> {
    let identity_index = get_identity_index(world, account).await.unwrap();

    let identity_value = world
        .read(&gdev::storage().identity().identities(identity_index))
        .await
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "indentity index {} does not have associated value",
                identity_index
            )
        })?;

    Ok(identity_value)
}
