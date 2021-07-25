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
use sp_storage::{ChildInfo, PrefixedStorageKey, StorageData, StorageKey};
use std::sync::Arc;

/// A client instance.
#[derive(Clone)]
pub enum Client {
    //G1(Arc<super::FullClient<g1_runtime::RuntimeApi, super::G1Executor>>),
    GTest(Arc<super::FullClient<gtest_runtime::RuntimeApi, super::GTestExecutor>>),
    GDev(Arc<super::FullClient<gdev_runtime::RuntimeApi, super::GDevExecutor>>),
}

impl sc_client_api::UsageProvider<Block> for Client {
    fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
        match self {
            Self::GTest(client) => client.usage_info(),
            Self::GDev(client) => client.usage_info(),
        }
    }
}

impl sc_client_api::BlockBackend<Block> for Client {
    fn block_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        match self {
            Self::GTest(client) => client.block_body(id),
            Self::GDev(client) => client.block_body(id),
        }
    }

    fn block_indexed_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        match self {
            Self::GTest(client) => client.block_indexed_body(id),
            Self::GDev(client) => client.block_indexed_body(id),
        }
    }

    fn block(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        match self {
            Self::GTest(client) => client.block(id),
            Self::GDev(client) => client.block(id),
        }
    }

    fn block_status(&self, id: &BlockId<Block>) -> sp_blockchain::Result<BlockStatus> {
        match self {
            Self::GTest(client) => client.block_status(id),
            Self::GDev(client) => client.block_status(id),
        }
    }

    fn justifications(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Justifications>> {
        match self {
            Self::GTest(client) => client.justifications(id),
            Self::GDev(client) => client.justifications(id),
        }
    }

    fn block_hash(
        &self,
        number: NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match self {
            Self::GTest(client) => client.block_hash(number),
            Self::GDev(client) => client.block_hash(number),
        }
    }

    fn indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        match self {
            Self::GTest(client) => client.indexed_transaction(hash),
            Self::GDev(client) => client.indexed_transaction(hash),
        }
    }

    fn has_indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<bool> {
        match self {
            Self::GTest(client) => client.has_indexed_transaction(hash),
            Self::GDev(client) => client.has_indexed_transaction(hash),
        }
    }
}

impl sc_client_api::StorageProvider<Block, super::FullBackend> for Client {
    fn storage(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match self {
            Self::GTest(client) => client.storage(id, key),
            Self::GDev(client) => client.storage(id, key),
        }
    }

    fn storage_keys(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match self {
            Self::GTest(client) => client.storage_keys(id, key_prefix),
            Self::GDev(client) => client.storage_keys(id, key_prefix),
        }
    }

    fn storage_hash(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match self {
            Self::GTest(client) => client.storage_hash(id, key),
            Self::GDev(client) => client.storage_hash(id, key),
        }
    }

    fn storage_pairs(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
        match self {
            Self::GTest(client) => client.storage_pairs(id, key_prefix),
            Self::GDev(client) => client.storage_pairs(id, key_prefix),
        }
    }

    fn storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <super::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match self {
            Self::GTest(client) => client.storage_keys_iter(id, prefix, start_key),
            Self::GDev(client) => client.storage_keys_iter(id, prefix, start_key),
        }
    }

    fn child_storage(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match self {
            Self::GTest(client) => client.child_storage(id, child_info, key),
            Self::GDev(client) => client.child_storage(id, child_info, key),
        }
    }

    fn child_storage_keys(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match self {
            Self::GTest(client) => client.child_storage_keys(id, child_info, key_prefix),
            Self::GDev(client) => client.child_storage_keys(id, child_info, key_prefix),
        }
    }

    fn child_storage_hash(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match self {
            Self::GTest(client) => client.child_storage_hash(id, child_info, key),
            Self::GDev(client) => client.child_storage_hash(id, child_info, key),
        }
    }

    fn max_key_changes_range(
        &self,
        first: NumberFor<Block>,
        last: BlockId<Block>,
    ) -> sp_blockchain::Result<Option<(NumberFor<Block>, BlockId<Block>)>> {
        match self {
            Self::GTest(client) => client.max_key_changes_range(first, last),
            Self::GDev(client) => client.max_key_changes_range(first, last),
        }
    }

    fn key_changes(
        &self,
        first: NumberFor<Block>,
        last: BlockId<Block>,
        storage_key: Option<&PrefixedStorageKey>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(NumberFor<Block>, u32)>> {
        match self {
            Self::GTest(client) => client.key_changes(first, last, storage_key, key),
            Self::GDev(client) => client.key_changes(first, last, storage_key, key),
        }
    }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, id: BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
        match self {
            Self::GTest(client) => client.header(&id),
            Self::GDev(client) => client.header(&id),
        }
    }
    fn info(&self) -> sp_blockchain::Info<Block> {
        match self {
            Self::GTest(client) => client.info(),
            Self::GDev(client) => client.info(),
        }
    }

    fn status(&self, id: BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        match self {
            Self::GTest(client) => client.status(id),
            Self::GDev(client) => client.status(id),
        }
    }

    fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
        match self {
            Self::GTest(client) => client.number(hash),
            Self::GDev(client) => client.number(hash),
        }
    }

    fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
        match self {
            Self::GTest(client) => client.hash(number),
            Self::GDev(client) => client.hash(number),
        }
    }
}

/// A set of APIs that runtimes must implement.
///
/// This trait has no methods or associated type. It is a concise marker for all the trait bounds
/// that it contains.
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_block_builder::BlockBuilder<Block>
    + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
    + pallet_grandpa::fg_primitives::GrandpaApi<Block>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + sp_api::Metadata<Block>
    + sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}
impl<Api> RuntimeApiCollection for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_block_builder::BlockBuilder<Block>
        + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
        + pallet_grandpa::fg_primitives::GrandpaApi<Block>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + sp_api::Metadata<Block>
        + sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}
