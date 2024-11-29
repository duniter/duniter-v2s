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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

pub mod parameters;
pub mod weights;

pub use self::parameters::*;
use common_runtime::IdtyNameValidatorImpl;
pub use common_runtime::{
    constants::*, entities::*, handlers::*, AccountId, Address, Balance, BlockNumber,
    FullIdentificationOfImpl, GetCurrentEpochIndex, Hash, Header, IdtyIndex, Index, Signature,
};
use frame_support::{traits::Contains, PalletId};
pub use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
pub use pallet_balances::Call as BalancesCall;
#[cfg(feature = "runtime-benchmarks")]
pub use pallet_collective::RawOrigin;
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as session_historical;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::FungibleAdapter;
pub use pallet_universal_dividend;
use scale_info::prelude::{vec, vec::Vec};
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perquintill,
};
pub use sp_runtime::{KeyTypeId, Perbill, Permill};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
pub use weights::paritydb_weights::constants::ParityDbWeight as DbWeight;

// A few exports that help ease life for downstream crates.
use frame_support::instances::Instance2;
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{EqualPrivilegeOnly, KeyOwnerProofSystem, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
        Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    StorageValue,
};

common_runtime::declare_session_keys! {}

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("gtest"),
    impl_name: create_runtime_str!("duniter-gtest"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 800,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// Block type as expected by this runtime.
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    pallet_oneshot_account::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

pub type TechnicalCommitteeInstance = Instance2;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [pallet_certification, Certification]
        [pallet_distance, Distance]
        [pallet_oneshot_account, OneshotAccount]
        [pallet_universal_dividend, UniversalDividend]
        [pallet_provide_randomness, ProvideRandomness]
        [pallet_upgrade_origin, UpgradeOrigin]
        [pallet_duniter_account, Account]
        [pallet_identity, Identity]
        [pallet_membership, Membership]
        [pallet_smith_members, SmithMembers]
        [pallet_authority_members, AuthorityMembers]
        // Substrate
        [pallet_balances, Balances]
        [frame_benchmarking::baseline, Baseline::<Runtime>]
        [pallet_collective, TechnicalCommittee]
        [pallet_session, SessionBench::<Runtime>]
        [pallet_im_online, ImOnline]
        [pallet_multisig, Multisig]
        [pallet_preimage, Preimage]
        [pallet_proxy, Proxy]
        [pallet_sudo, Sudo]
        [pallet_scheduler, Scheduler]
        [frame_system, SystemBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_treasury, Treasury]
        [pallet_utility, Utility]
    );
}

pub struct BaseCallFilter;
impl Contains<RuntimeCall> for BaseCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        !matches!(call, RuntimeCall::Session(_))
    }
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    codec::Encode,
    codec::Decode,
    frame_support::pallet_prelude::RuntimeDebug,
    codec::MaxEncodedLen,
    scale_info::TypeInfo,
)]
#[allow(clippy::unnecessary_cast)]
pub enum ProxyType {
    AlmostAny = 0,
    TransferOnly = 1,
    CancelProxy = 2,
    TechnicalCommitteePropose = 3,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::AlmostAny
    }
}
impl frame_support::traits::InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::AlmostAny => {
                // Some calls are never authorized from a proxied account
                !matches!(
                    c,
                    RuntimeCall::Certification(..)
                        | RuntimeCall::Identity(..)
                        | RuntimeCall::SmithMembers(..)
                )
            }
            ProxyType::TransferOnly => {
                matches!(
                    c,
                    RuntimeCall::Balances(..) | RuntimeCall::UniversalDividend(..)
                )
            }
            ProxyType::CancelProxy => {
                matches!(
                    c,
                    RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
                )
            }
            ProxyType::TechnicalCommitteePropose => {
                matches!(
                    c,
                    RuntimeCall::TechnicalCommittee(pallet_collective::Call::propose { .. })
                )
            }
        }
    }
}

// Configure pallets to include in runtime.
#[cfg(feature = "runtime-benchmarks")]
type WorstOrigin = RawOrigin<AccountId, TechnicalCommitteeInstance>;
common_runtime::pallets_config!();

// Create the runtime by composing the pallets that were previously configured.
construct_runtime!(
    pub enum Runtime
    {
        // Basic stuff
        System: frame_system = 0,
        Account: pallet_duniter_account = 1,
        Scheduler: pallet_scheduler = 2,

        // Block creation
        Babe: pallet_babe = 3,
        Timestamp: pallet_timestamp = 4,

        // Money management
        Balances: pallet_balances = 6,
        TransactionPayment: pallet_transaction_payment = 32,
        OneshotAccount: pallet_oneshot_account = 7,
        Quota: pallet_quota = 66,

        // Consensus support
        SmithMembers: pallet_smith_members = 10,
        AuthorityMembers: pallet_authority_members = 11,
        Authorship: pallet_authorship = 12,
        Offences: pallet_offences = 13,
        Historical: session_historical = 14,
        Session: pallet_session = 15,
        Grandpa: pallet_grandpa= 16,
        ImOnline: pallet_im_online = 17,
        AuthorityDiscovery: pallet_authority_discovery = 18,

        // Governance stuff
        Sudo: pallet_sudo = 20,
        UpgradeOrigin: pallet_upgrade_origin = 21,
        Preimage: pallet_preimage = 22,
        TechnicalCommittee: pallet_collective::<Instance2> = 23,

        // Universal dividend
        UniversalDividend: pallet_universal_dividend = 30,

        // Web Of Trust
        Wot: pallet_duniter_wot = 40,
        Identity: pallet_identity = 41,
        Membership: pallet_membership = 42,
        Certification: pallet_certification = 43,
        Distance: pallet_distance = 44,

        // Utilities
        AtomicSwap: pallet_atomic_swap = 50,
        Multisig: pallet_multisig = 51,
        ProvideRandomness: pallet_provide_randomness = 52,
        Proxy: pallet_proxy = 53,
        Utility: pallet_utility = 54,
        Treasury: pallet_treasury = 55,
    }
);

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}

// All of our runtimes share most of their Runtime API implementations.
// We use a macro to implement this common part and add runtime-specific additional implementations.
// This macro expands to :
// ```
// impl_runtime_apis! {
//     // All impl blocks shared between all runtimes.
//
//     // Specific impls provided to the `runtime_apis!` macro.
// }
// ```
common_runtime::runtime_apis! {}
