// Copyright 2023 Axiom-Team
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

use crate::runtime;

use sp_core::H256;
use subxt::storage::StorageKey;

pub type Client = subxt::OnlineClient<crate::RuntimeConfig>;
pub type AccountId = subxt::ext::sp_runtime::AccountId32;
pub type IdtyIndex = u32;

pub async fn client(rpc_url: String) -> Client {
    Client::from_url(rpc_url)
        .await
        .expect("Cannot create RPC client")
}

pub async fn parent_hash(client: &Client) -> H256 {
    client
        .storage()
        .fetch(&runtime::storage().system().parent_hash(), None)
        .await
        .expect("Cannot fetch parent hash")
        .expect("Parent hash is None")
}

pub async fn current_session(client: &Client, parent_hash: H256) -> u32 {
    client
        .storage()
        .fetch(
            &runtime::storage().session().current_index(),
            Some(parent_hash),
        )
        .await
        .expect("Cannot fetch current session")
        .unwrap_or_default()
}

pub async fn current_pool(
    client: &Client,
    parent_hash: H256,
    current_session: u32,
) -> Option<runtime::runtime_types::pallet_distance::types::EvaluationPool<AccountId, IdtyIndex>> {
    client
        .storage()
        .fetch(
            &match current_session % 3 {
                0 => runtime::storage().distance().evaluation_pool1(),
                1 => runtime::storage().distance().evaluation_pool2(),
                2 => runtime::storage().distance().evaluation_pool0(),
                _ => unreachable!("n%3<3"),
            },
            Some(parent_hash),
        )
        .await
        .expect("Cannot fetch current pool")
}

pub async fn evaluation_block(client: &Client, parent_hash: H256) -> H256 {
    client
        .storage()
        .fetch(
            &runtime::storage().distance().evaluation_block(),
            Some(parent_hash),
        )
        .await
        .expect("Cannot fetch evaluation block")
        .expect("No evaluation block")
}

pub async fn member_iter(client: &Client, evaluation_block: H256) -> MemberIter {
    MemberIter(
        client
            .storage()
            .iter(
                runtime::storage().membership().membership(0),
                100,
                Some(evaluation_block),
            )
            .await
            .expect("Cannot fetch memberships"),
    )
}

pub struct MemberIter(
    subxt::storage::KeyIter<
        crate::RuntimeConfig,
        Client,
        subxt::metadata::DecodeStaticType<
            runtime::runtime_types::sp_membership::MembershipData<u32>,
        >,
    >,
);

impl MemberIter {
    pub async fn next(&mut self) -> Result<Option<IdtyIndex>, subxt::error::Error> {
        Ok(self
            .0
            .next()
            .await?
            .map(|(storage_key, _membership_data)| idty_id_from_storage_key(&storage_key)))
    }
}

pub async fn cert_iter(client: &Client, evaluation_block: H256) -> CertIter {
    CertIter(
        client
            .storage()
            .iter(
                runtime::storage().cert().certs_by_receiver(0),
                100,
                Some(evaluation_block),
            )
            .await
            .expect("Cannot fetch certifications"),
    )
}

pub struct CertIter(
    subxt::storage::KeyIter<
        crate::RuntimeConfig,
        Client,
        subxt::metadata::DecodeStaticType<Vec<(IdtyIndex, u32)>>,
    >,
);

impl CertIter {
    pub async fn next(
        &mut self,
    ) -> Result<Option<(IdtyIndex, Vec<(IdtyIndex, u32)>)>, subxt::error::Error> {
        Ok(self
            .0
            .next()
            .await?
            .map(|(storage_key, issuers)| (idty_id_from_storage_key(&storage_key), issuers)))
    }
}

fn idty_id_from_storage_key(storage_key: &StorageKey) -> IdtyIndex {
    u32::from_le_bytes(storage_key.as_ref()[40..44].try_into().unwrap())
}
