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

use crate::runtime::runtime_types::{
    pallet_distance::median::MedianAcc, sp_arithmetic::per_things::Perbill,
};

use dubp_wot::{data::rusty::RustyWebOfTrust, WebOfTrust, WotId};
use sp_core::H256;
use std::collections::BTreeSet;

pub struct Client {
    wot: RustyWebOfTrust,
    pub pool_len: usize,
}
pub type AccountId = subxt::ext::sp_runtime::AccountId32;
pub type IdtyIndex = u32;

pub struct EvaluationPool<AccountId: Ord, IdtyIndex> {
    pub evaluations: (Vec<(IdtyIndex, MedianAcc<Perbill>)>,),
    pub evaluators: BTreeSet<AccountId>,
}

pub async fn client(_rpc_url: impl AsRef<str>) -> Client {
    unimplemented!()
}

pub fn client_from_wot(wot: RustyWebOfTrust) -> Client {
    Client { wot, pool_len: 1 }
}

pub async fn parent_hash(_client: &Client) -> H256 {
    Default::default()
}

pub async fn current_pool_index(_client: &Client, _parent_hash: H256) -> u32 {
    0
}

pub async fn current_pool(
    client: &Client,
    _parent_hash: H256,
    _current_session: u32,
) -> Option<EvaluationPool<AccountId, IdtyIndex>> {
    Some(EvaluationPool {
        evaluations: (client
            .wot
            .get_enabled()
            .into_iter()
            .chain(client.wot.get_disabled().into_iter())
            .zip(0..client.pool_len)
            .map(|(wot_id, _)| {
                (wot_id.0 as IdtyIndex, unsafe {
                    std::mem::transmute::<
                        (std::vec::Vec<()>, std::option::Option<u32>, i32),
                        MedianAcc<Perbill>,
                    >((Vec::<()>::new(), Option::<u32>::None, 0))
                })
            })
            .collect(),),
        evaluators: BTreeSet::new(),
    })
}

pub async fn evaluation_block(_client: &Client, _parent_hash: H256) -> H256 {
    Default::default()
}

pub async fn max_referee_distance(_client: &Client) -> u32 {
    5
}

pub async fn member_iter(client: &Client, _evaluation_block: H256) -> MemberIter {
    MemberIter(client.wot.get_enabled().into_iter())
}

pub struct MemberIter(std::vec::IntoIter<WotId>);

impl MemberIter {
    pub async fn next(&mut self) -> Result<Option<IdtyIndex>, subxt::error::Error> {
        Ok(self.0.next().map(|wot_id| wot_id.0 as u32))
    }
}

pub async fn cert_iter(client: &Client, _evaluation_block: H256) -> CertIter {
    CertIter(
        client
            .wot
            .get_enabled()
            .iter()
            .chain(client.wot.get_disabled().iter())
            .map(|wot_id| {
                (
                    wot_id.0 as IdtyIndex,
                    client
                        .wot
                        .get_links_source(*wot_id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|wot_id| (wot_id.0 as IdtyIndex, 0))
                        .collect::<Vec<(IdtyIndex, u32)>>(),
                )
            })
            .collect::<Vec<_>>()
            .into_iter(),
    )
}

pub struct CertIter(std::vec::IntoIter<(IdtyIndex, Vec<(IdtyIndex, u32)>)>);

impl CertIter {
    pub async fn next(&mut self) -> Result<Option<(u32, Vec<(u32, u32)>)>, subxt::error::Error> {
        Ok(self.0.next())
    }
}
