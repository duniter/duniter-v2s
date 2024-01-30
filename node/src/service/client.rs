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
use sc_client_api::MerkleValue;
use sc_client_api::{
    AuxStore, Backend as BackendT, BlockchainEvents, KeysIter, PairsIter, UsageProvider,
};
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::BlockStatus;
use sp_core::{Encode, Pair};
use sp_runtime::{
    generic::SignedBlock,
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
    + CallApiAt<Block>
    + AuxStore
    + UsageProvider<Block>
    + HeaderMetadata<Block, Error = sp_blockchain::Error>
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sc_client_api::StateBackend<BlakeTwo256>,
    Self::Api: RuntimeApiCollection,
{
}

impl<Block, Backend, Client> AbstractClient<Block, Backend> for Client
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sc_client_api::StateBackend<BlakeTwo256>,
    Client: BlockchainEvents<Block>
        + ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + UsageProvider<Block>
        + Sized
        + Send
        + Sync
        + CallApiAt<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>,
    Client::Api: RuntimeApiCollection,
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
        Backend: sc_client_api::Backend<Block> + 'static,
        Backend::State: sc_client_api::StateBackend<BlakeTwo256>,
        Api: crate::service::RuntimeApiCollection,
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
{
}
impl<Api> RuntimeApiCollection for Api where
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
        + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
{
}

/// A client instance.
#[derive(Clone)]
pub enum Client {
    #[cfg(feature = "g1")]
    G1(Arc<super::FullClient<g1_runtime::RuntimeApi>>),
    #[cfg(feature = "gtest")]
    GTest(Arc<super::FullClient<gtest_runtime::RuntimeApi>>),
    #[cfg(feature = "gdev")]
    GDev(Arc<super::FullClient<gdev_runtime::RuntimeApi>>),
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
            Self::G1($client) => {
                #[allow(unused_imports)]
                use g1_runtime as runtime;
                $( $code )*
            }
            #[cfg(feature = "gtest")]
            Self::GTest($client) => {
                #[allow(unused_imports)]
                use gtest_runtime as runtime;
                $( $code )*
            }
            #[cfg(feature = "gdev")]
            Self::GDev($client) => {
                #[allow(unused_imports)]
                use gdev_runtime as runtime;
                $( $code )*
            }
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
impl From<Arc<super::FullClient<g1_runtime::RuntimeApi>>> for Client {
    fn from(client: Arc<super::FullClient<g1_runtime::RuntimeApi>>) -> Self {
        Self::G1(client)
    }
}

#[cfg(feature = "gtest")]
impl From<Arc<super::FullClient<gtest_runtime::RuntimeApi>>> for Client {
    fn from(client: Arc<super::FullClient<gtest_runtime::RuntimeApi>>) -> Self {
        Self::GTest(client)
    }
}

#[cfg(feature = "gdev")]
impl From<Arc<super::FullClient<gdev_runtime::RuntimeApi>>> for Client {
    fn from(client: Arc<super::FullClient<gdev_runtime::RuntimeApi>>) -> Self {
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
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        match_client!(self, block_body(hash))
    }

    fn block_indexed_body(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        match_client!(self, block_indexed_body(hash))
    }

    fn block(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        match_client!(self, block(hash))
    }

    fn block_status(&self, hash: <Block as BlockT>::Hash) -> sp_blockchain::Result<BlockStatus> {
        match_client!(self, block_status(hash))
    }

    fn justifications(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Justifications>> {
        match_client!(self, justifications(hash))
    }

    fn block_hash(
        &self,
        number: sp_runtime::traits::NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, block_hash(number))
    }

    fn indexed_transaction(
        &self,
        id: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        match_client!(self, indexed_transaction(id))
    }

    /*fn has_indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<bool> {
        match_client!(self, has_indexed_transaction(hash))
    }*/

    fn requires_full_sync(&self) -> bool {
        match_client!(self, requires_full_sync())
    }
}

/// Helper trait to implement [`frame_benchmarking_cli::ExtrinsicBuilder`].
///
/// Should only be used for benchmarking since it makes strong assumptions
/// about the chain state that these calls will be valid for.
trait BenchmarkCallSigner<RuntimeCall: Encode + Clone, Signer: Pair> {
    /// Signs a call together with the signed extensions of the specific runtime.
    ///
    /// Only works if the current block is the genesis block since the
    /// `CheckMortality` check is mocked by using the genesis block.
    fn sign_call(
        &self,
        call: RuntimeCall,
        nonce: u32,
        current_block: u64,
        period: u64,
        genesis: sp_core::H256,
        acc: Signer,
    ) -> sp_runtime::OpaqueExtrinsic;
}

#[cfg(feature = "g1")]
use g1_runtime as runtime;
#[cfg(feature = "gdev")]
use gdev_runtime as runtime;
#[cfg(feature = "gdev")]
type FullClient = super::FullClient<runtime::RuntimeApi>;
#[cfg(feature = "gtest")]
use gtest_runtime as runtime;
#[cfg(feature = "gtest")]
type FullClient = super::FullClient<runtime::RuntimeApi>;

#[cfg(any(feature = "gdev", feature = "gtest"))]
impl BenchmarkCallSigner<runtime::RuntimeCall, sp_core::sr25519::Pair> for FullClient {
    fn sign_call(
        &self,
        call: runtime::RuntimeCall,
        nonce: u32,
        current_block: u64,
        period: u64,
        genesis: sp_core::H256,
        acc: sp_core::sr25519::Pair,
    ) -> sp_runtime::OpaqueExtrinsic {
        // use runtime;

        let extra: runtime::SignedExtra = (
            frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
            frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
            frame_system::CheckTxVersion::<runtime::Runtime>::new(),
            frame_system::CheckGenesis::<runtime::Runtime>::new(),
            frame_system::CheckMortality::<runtime::Runtime>::from(
                sp_runtime::generic::Era::mortal(period, current_block),
            ),
            frame_system::CheckNonce::<runtime::Runtime>::from(nonce).into(),
            frame_system::CheckWeight::<runtime::Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        );

        let payload = sp_runtime::generic::SignedPayload::from_raw(
            call.clone(),
            extra.clone(),
            (
                (),
                runtime::VERSION.spec_version,
                runtime::VERSION.transaction_version,
                genesis,
                genesis,
                (),
                (),
                (),
            ),
        );

        let signature = payload.using_encoded(|p| acc.sign(p));
        runtime::UncheckedExtrinsic::new_signed(
            call,
            sp_runtime::AccountId32::from(acc.public()).into(),
            common_runtime::Signature::Sr25519(signature),
            extra,
        )
        .into()
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for Client {
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> std::result::Result<sp_runtime::OpaqueExtrinsic, &'static str> {
        with_client! {
            self, client, {
                let call = runtime::RuntimeCall::System(runtime::SystemCall::remark { remark: vec![] });
                let signer = sp_keyring::Sr25519Keyring::Bob.pair();

                let period = runtime::BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
                let genesis = client.usage_info().chain.best_hash;

                Ok(client.sign_call(call, nonce, 0, period, genesis, signer))
            }
        }
    }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, hash: Hash) -> sp_blockchain::Result<Option<Header>> {
        match_client!(self, header(hash))
    }

    fn info(&self) -> sp_blockchain::Info<Block> {
        match_client!(self, info())
    }

    fn status(&self, hash: Hash) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        match_client!(self, status(hash))
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
        hash: <Block as BlockT>::Hash,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, storage(hash, key))
    }

    fn storage_keys(
        &self,
        hash: <Block as BlockT>::Hash,
        key_prefix: Option<&StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeysIter<<super::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(self, storage_keys(hash, key_prefix, start_key))
    }

    fn storage_hash(
        &self,
        hash: <Block as BlockT>::Hash,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, storage_hash(hash, key))
    }

    fn storage_pairs(
        &self,
        hash: <Block as BlockT>::Hash,
        key_prefix: Option<&StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        PairsIter<<super::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(self, storage_pairs(hash, key_prefix, start_key))
    }

    fn child_storage(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, child_storage(hash, child_info, key))
    }

    fn child_storage_keys(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: ChildInfo,
        key_prefix: Option<&StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeysIter<<super::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(
            self,
            child_storage_keys(hash, child_info, key_prefix, start_key)
        )
    }

    fn child_storage_hash(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, child_storage_hash(hash, child_info, key))
    }

    // Given a block's hash and a key, return the closest merkle value.
    fn closest_merkle_value(
        &self,
        hash: <Block as BlockT>::Hash,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<MerkleValue<<Block as BlockT>::Hash>>> {
        match_client!(self, closest_merkle_value(hash, key))
    }

    // Given a block's hash and a key and a child storage key, return the closest merkle value.
    fn child_closest_merkle_value(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<MerkleValue<<Block as BlockT>::Hash>>> {
        match_client!(self, child_closest_merkle_value(hash, child_info, key))
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
#[cfg(feature = "runtime-benchmarks")]
pub fn benchmark_inherent_data(
) -> std::result::Result<sp_inherents::InherentData, sp_inherents::Error> {
    use sp_inherents::InherentDataProvider;
    let mut inherent_data = sp_inherents::InherentData::new();

    // Assume that all runtimes have the `timestamp` pallet.
    let d = std::time::Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());
    futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))?;

    Ok(inherent_data)
}
