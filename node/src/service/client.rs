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

use common_runtime::{AccountId, Balance, Block, BlockNumber, Hash, Header, Index};
use sc_client_api::KeyIterator;
use sp_api::NumberFor;
use sp_consensus::BlockStatus;
use sp_runtime::{
    generic::{BlockId, SignedBlock},
    traits::{BlakeTwo256, Block as BlockT},
    Justifications,
};
use sp_storage::{ChildInfo, StorageData, StorageKey};
use std::sync::Arc;

/// A client instance.
#[derive(Clone)]
pub enum Client {
    #[cfg(feature = "g1")]
    G1(Arc<super::FullClient<g1_runtime::RuntimeApi, super::G1Executor>>),
    #[cfg(feature = "gtest")]
    GTest(Arc<super::FullClient<gtest_runtime::RuntimeApi, super::GTestExecutor>>),
    #[cfg(feature = "gdev")]
    GDev(Arc<super::FullClient<gdev_runtime::RuntimeApi, super::GDevExecutor>>),
}

#[cfg(feature = "g1")]
impl From<Arc<super::FullClient<g1_runtime::RuntimeApi, super::G1Executor>>> for Client {
    fn from(client: Arc<super::FullClient<g1_runtime::RuntimeApi, super::G1Executor>>) -> Self {
        Self::G1(client)
    }
}

#[cfg(feature = "gtest")]
impl From<Arc<super::FullClient<gtest_runtime::RuntimeApi, super::GTestExecutor>>> for Client {
    fn from(
        client: Arc<super::FullClient<gtest_runtime::RuntimeApi, super::GTestExecutor>>,
    ) -> Self {
        Self::GTest(client)
    }
}

#[cfg(feature = "gdev")]
impl From<Arc<super::FullClient<gdev_runtime::RuntimeApi, super::GDevExecutor>>> for Client {
    fn from(client: Arc<super::FullClient<gdev_runtime::RuntimeApi, super::GDevExecutor>>) -> Self {
        Self::GDev(client)
    }
}

macro_rules! match_client {
    ($self:ident, $method:ident($($param:ident),*)) => {
        match $self {
            #[cfg(feature = "g1")]
            Self::G1(client) => client.$method($($param),*),
            #[cfg(feature = "gtest")]
            Self::GTest(client) => client.$method($($param),*),
            #[cfg(feature = "gdev")]
            Self::GDev(client) => client.$method($($param),*),
        }
    };
}

impl sc_client_api::UsageProvider<Block> for Client {
    fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
        match_client!(self, usage_info())
    }
}

impl sc_client_api::BlockBackend<Block> for Client {
    fn block_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        match_client!(self, block_body(id))
    }

    fn block_indexed_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        match_client!(self, block_indexed_body(id))
    }

    fn block(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        match_client!(self, block(id))
    }

    fn block_status(&self, id: &BlockId<Block>) -> sp_blockchain::Result<BlockStatus> {
        match_client!(self, block_status(id))
    }

    fn justifications(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Justifications>> {
        match_client!(self, justifications(id))
    }

    fn block_hash(
        &self,
        number: NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, block_hash(number))
    }

    fn indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        match_client!(self, indexed_transaction(hash))
    }

    fn has_indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<bool> {
        match_client!(self, has_indexed_transaction(hash))
    }
}

impl sc_client_api::StorageProvider<Block, super::FullBackend> for Client {
    fn storage(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, storage(id, key))
    }

    fn storage_keys(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match_client!(self, storage_keys(id, key_prefix))
    }

    fn storage_hash(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, storage_hash(id, key))
    }

    fn storage_pairs(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
        match_client!(self, storage_pairs(id, key_prefix))
    }

    fn storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <super::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(self, storage_keys_iter(id, prefix, start_key))
    }

    fn child_storage(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, child_storage(id, child_info, key))
    }

    fn child_storage_keys(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match_client!(self, child_storage_keys(id, child_info, key_prefix))
    }

    fn child_storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        child_info: ChildInfo,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <super::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(
            self,
            child_storage_keys_iter(id, child_info, prefix, start_key)
        )
    }

    fn child_storage_hash(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, child_storage_hash(id, child_info, key))
    }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, id: BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
        let id = &id;
        match_client!(self, header(id))
    }
    fn info(&self) -> sp_blockchain::Info<Block> {
        match_client!(self, info())
    }

    fn status(&self, id: BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        match_client!(self, status(id))
    }

    fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
        match_client!(self, number(hash))
    }

    fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
        match_client!(self, hash(number))
    }
}

/// A set of APIs that runtimes must implement.
///
/// This trait has no methods or associated type. It is a concise marker for all the trait bounds
/// that it contains.
pub trait RuntimeApiCollection:
    pallet_grandpa::fg_primitives::GrandpaApi<Block>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + sp_api::ApiExt<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + sp_api::Metadata<Block>
    + sp_consensus_babe::BabeApi<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}
impl<Api> RuntimeApiCollection for Api
where
    Api: pallet_grandpa::fg_primitives::GrandpaApi<Block>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + sp_api::ApiExt<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_api::Metadata<Block>
        + sp_consensus_babe::BabeApi<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}
