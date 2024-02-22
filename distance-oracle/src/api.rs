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

#![allow(clippy::type_complexity)]

use crate::runtime;
use log::debug;

use sp_core::H256;

pub type Client = subxt::OnlineClient<crate::RuntimeConfig>;
pub type AccountId = subxt::utils::AccountId32;
pub type IdtyIndex = u32;

pub async fn client(rpc_url: String) -> Client {
    Client::from_insecure_url(rpc_url)
        .await
        .expect("Cannot create RPC client")
}

pub async fn parent_hash(client: &Client) -> H256 {
    client
        .blocks()
        .at_latest()
        .await
        .expect("Cannot fetch latest block hash")
        .hash()
}

pub async fn current_session(client: &Client, parent_hash: H256) -> u32 {
    client
        .storage()
        .at(parent_hash)
        .fetch(&runtime::storage().session().current_index())
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
        .at(parent_hash)
        .fetch(&match current_session % 3 {
            0 => {
                debug!("Looking at Pool1 for session {}", current_session);
                runtime::storage().distance().evaluation_pool1()
            }
            1 => {
                debug!("Looking at Pool2 for session {}", current_session);
                runtime::storage().distance().evaluation_pool2()
            }
            2 => {
                debug!("Looking at Pool0 for session {}", current_session);
                runtime::storage().distance().evaluation_pool0()
            }
            _ => unreachable!("n%3<3"),
        })
        .await
        .expect("Cannot fetch current pool")
}

pub async fn evaluation_block(client: &Client, parent_hash: H256) -> H256 {
    client
        .storage()
        .at(parent_hash)
        .fetch(&runtime::storage().distance().evaluation_block())
        .await
        .expect("Cannot fetch evaluation block")
        .expect("No evaluation block")
}

pub async fn max_referee_distance(client: &Client) -> u32 {
    client
        .constants()
        .at(&runtime::constants().distance().max_referee_distance())
        .expect("Cannot fetch referee distance")
}

pub async fn member_iter(client: &Client, evaluation_block: H256) -> MemberIter {
    MemberIter(
        client
            .storage()
            .at(evaluation_block)
            .iter(runtime::storage().membership().membership_iter())
            .await
            .expect("Cannot fetch memberships"),
    )
}

pub struct MemberIter(
    subxt::backend::StreamOfResults<(
        Vec<u8>,
        runtime::runtime_types::sp_membership::MembershipData<u32>,
    )>,
);

impl MemberIter {
    pub async fn next(&mut self) -> Result<Option<IdtyIndex>, subxt::error::Error> {
        self.0.next().await.transpose().map(|i| {
            i.map(|(storage_key, _membership_data)| idty_id_from_storage_key(&storage_key))
        })
    }
}

pub async fn cert_iter(client: &Client, evaluation_block: H256) -> CertIter {
    CertIter(
        client
            .storage()
            .at(evaluation_block)
            .iter(runtime::storage().certification().certs_by_receiver_iter())
            .await
            .expect("Cannot fetch certifications"),
    )
}

pub struct CertIter(subxt::backend::StreamOfResults<(Vec<u8>, Vec<(IdtyIndex, u32)>)>);

impl CertIter {
    pub async fn next(
        &mut self,
    ) -> Result<Option<(IdtyIndex, Vec<(IdtyIndex, u32)>)>, subxt::error::Error> {
        self.0.next().await.transpose().map(|i| {
            i.map(|(storage_key, issuers)| (idty_id_from_storage_key(&storage_key), issuers))
        })
    }
}

fn idty_id_from_storage_key(storage_key: &[u8]) -> IdtyIndex {
    u32::from_le_bytes(
        storage_key[40..44]
            .try_into()
            .expect("Cannot convert StorageKey to IdtyIndex"),
    )
}
