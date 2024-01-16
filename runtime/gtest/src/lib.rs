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

pub use self::parameters::*;
pub use common_runtime::{
    constants::*, entities::*, handlers::*, AccountId, Address, Balance, BlockNumber,
    FullIdentificationOfImpl, GetCurrentEpochIndex, Hash, Header, IdtyIndex, Index, Signature,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as session_historical;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
pub use pallet_universal_dividend;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{KeyTypeId, Perbill, Permill};

use common_runtime::IdtyNameValidatorImpl;
use frame_support::traits::Contains;
use frame_support::PalletId;
use frame_system::EnsureRoot;
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::traits::{
    AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, One, OpaqueKeys,
};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
use frame_support::instances::Instance2;
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{EqualPrivilegeOnly, KeyOwnerProofSystem, Randomness},
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
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
    spec_version: 701,
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
        [pallet_scheduler, Scheduler]
        [frame_system, SystemBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_treasury, Treasury]
        [pallet_utility, Utility]
    );
}

pub struct BaseCallFilter;

// implement filter
// session pallet calls are filtered in order to use authority member instead
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
    frame_support::RuntimeDebug,
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
common_runtime::pallets_config! {

    impl pallet_sudo::Config for Runtime {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeCall = RuntimeCall;
    }
}

// Create the runtime by composing the pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = common_runtime::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Account: pallet_duniter_account::{Pallet, Storage, Config<T>, Event<T>} = 1,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 2,

        // Block creation
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 3,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 4,

        // Money management
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 32,
        OneshotAccount: pallet_oneshot_account::{Pallet, Call, Storage, Event<T>} = 7,
        Quota: pallet_quota::{Pallet, Storage, Config<T>, Event<T>} = 66,

        // Consensus support
        SmithMembers: pallet_smith_members::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        AuthorityMembers: pallet_authority_members::{Pallet, Call, Storage, Config<T>, Event<T>} = 11,
        Authorship: pallet_authorship::{Pallet, Storage} = 12,
        Offences: pallet_offences::{Pallet, Storage, Event} = 13,
        Historical: session_historical::{Pallet} = 14,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 15,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 16,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 17,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 18,

        // Governance stuff
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 20,
        UpgradeOrigin: pallet_upgrade_origin::{Pallet, Call, Event} = 21,
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 22,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>, Origin<T>} = 23,

        // Universal dividend
        UniversalDividend: pallet_universal_dividend::{Pallet, Call, Config<T>, Storage, Event<T>} = 30,

        // Web Of Trust
        Wot: pallet_duniter_wot::{Pallet} = 40,
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>} = 41,
        Membership: pallet_membership::{Pallet, Call, Config<T>, Storage, Event<T>} = 42,
        Certification: pallet_certification::{Pallet, Call, Config<T>, Storage, Event<T>} = 43,
        Distance: pallet_distance::{Pallet, Call, Storage, Inherent, Event<T>} = 44,

        // Utilities
        AtomicSwap: pallet_atomic_swap::{Pallet, Call, Storage, Event<T>} = 50,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 51,
        ProvideRandomness: pallet_provide_randomness::{Pallet, Call, Storage, Event} = 52,
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 53,
        Utility: pallet_utility::{Pallet, Call, Event} = 54,
        Treasury: pallet_treasury::{Pallet, Call, Config, Storage, Event<T>} = 55,
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
