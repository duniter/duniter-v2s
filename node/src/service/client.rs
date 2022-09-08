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

use common_runtime::{AccountId, Balance, Block, BlockNumber, Hash, Header, Index};
use sc_client_api::{AuxStore, Backend as BackendT, BlockchainEvents, KeyIterator, UsageProvider};
use sp_api::{CallApiAt, NumberFor, ProvideRuntimeApi};
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::BlockStatus;
use sp_runtime::{
    generic::{BlockId, SignedBlock},
    traits::{BlakeTwo256, Block as BlockT},
    Justifications,
};
use sp_storage::{ChildInfo, StorageData, StorageKey};
use std::sync::Arc;

/// Trait that abstracts over all available client implementations.
///
/// For a concrete type there exists [`Client`].
pub trait AbstractClient<Block, Backend>:
    BlockchainEvents<Block>
    + Sized
    + Send
    + Sync
    + ProvideRuntimeApi<Block>
    + HeaderBackend<Block>
    + CallApiAt<Block, StateBackend = Backend::State>
    + AuxStore
    + UsageProvider<Block>
    + HeaderMetadata<Block, Error = sp_blockchain::Error>
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Self::Api: RuntimeApiCollection<StateBackend = Backend::State>,
{
}

impl<Block, Backend, Client> AbstractClient<Block, Backend> for Client
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Client: BlockchainEvents<Block>
        + ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + UsageProvider<Block>
        + Sized
        + Send
        + Sync
        + CallApiAt<Block, StateBackend = Backend::State>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>,
    Client::Api: RuntimeApiCollection<StateBackend = Backend::State>,
{
}

/// A handle to a client instance.
///
/// The service supports multiple different runtimes (gtest, g1 itself, etc). As each runtime has a
/// specialized client, we need to hide them behind a trait. This is this trait.
///
/// When wanting to work with the inner client, you need to use `execute_with`.
///
/// See [`ExecuteWithClient`](trait.ExecuteWithClient.html) for more information.
pub trait ClientHandle {
    /// Execute the given something with the client.
    fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output;
}

/// Execute something with the client instance.
///
/// As there exist multiple chains, like g1 itself, gtest, gdev etc,
/// there can exist different kinds of client types. As these client types differ in the generics
/// that are being used, we can not easily return them from a function. For returning them from a
/// function there exists [`Client`]. However, the problem on how to use this client instance still
/// exists. This trait "solves" it in a dirty way. It requires a type to implement this trait and
/// than the [`execute_with_client`](ExecuteWithClient::execute_with_client) function can be called
/// with any possible client instance.
///
/// In a perfect world, we could make a closure work in this way.
pub trait ExecuteWithClient {
    /// The return type when calling this instance.
    type Output;

    /// Execute whatever should be executed with the given client instance.
    fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
    where
        <Api as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        Backend: sc_client_api::Backend<Block> + 'static,
        Backend::State: sp_api::StateBackend<BlakeTwo256>,
        Api: crate::service::RuntimeApiCollection<StateBackend = Backend::State>,
        Client: AbstractClient<Block, Backend, Api = Api> + 'static;
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

macro_rules! with_client {
	{
		$self:ident,
		$client:ident,
		{
			$( $code:tt )*
		}
	} => {
		match $self {
			#[cfg(feature = "g1")]
			Self::G1($client) => { $( $code )* },
			#[cfg(feature = "gtest")]
			Self::GTest($client) => { $( $code )* },
			#[cfg(feature = "gdev")]
			Self::GDev($client) => { $( $code )* },
		}
	}
}

impl ClientHandle for Client {
    fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output {
        with_client! {
            self,
            client,
            {
                T::execute_with_client::<_, _, super::FullBackend>(t, client.clone())
            }
        }
    }
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

    fn requires_full_sync(&self) -> bool {
        match_client!(self, requires_full_sync())
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for Client {
    fn remark(
        &self,
        _nonce: u32,
    ) -> std::result::Result<sp_runtime::OpaqueExtrinsic, &'static str> {
        todo!()
        /*with_signed_payload! {
            self,
            {extra, client, raw_payload},
            {
                // First the setup code to init all the variables that are needed
                // to build the signed extras.
                use runtime::{Call, SystemCall};

                let call = Call::System(SystemCall::remark { remark: vec![] });
                let bob = Sr25519Keyring::Bob.pair();

                let period = polkadot_runtime_common::BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;

                let current_block = 0;
                let tip = 0;
                let genesis = client.usage_info().chain.best_hash;
            },
            (period, current_block, nonce, tip, call, genesis),
            /* The SignedPayload is generated here */
            {
                // Use the payload to generate a signature.
                let signature = raw_payload.using_encoded(|payload| bob.sign(payload));

                let ext = runtime::UncheckedExtrinsic::new_signed(
                    call,
                    sp_runtime::AccountId32::from(bob.public()).into(),
                    polkadot_core_primitives::Signature::Sr25519(signature.clone()),
                    extra,
                );
                Ok(ext.into())
            }
        }*/
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

impl sc_client_api::UsageProvider<Block> for Client {
    fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
        match_client!(self, usage_info())
    }
}

/// Generates inherent data for benchmarking G1, GTest and GDev.
///
/// Not to be used outside of benchmarking since it returns mocked values.
pub fn benchmark_inherent_data(
) -> std::result::Result<sp_inherents::InherentData, sp_inherents::Error> {
    use sp_inherents::InherentDataProvider;
    let mut inherent_data = sp_inherents::InherentData::new();

    // Assume that all runtimes have the `timestamp` pallet.
    let d = std::time::Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());
    timestamp.provide_inherent_data(&mut inherent_data)?;

    Ok(inherent_data)
}
