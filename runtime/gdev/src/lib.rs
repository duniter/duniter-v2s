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
pub use pallet_duniter_test_parameters::Parameters as GenesisParameters;
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
    generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Cow, Perquintill,
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

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("gdev"),
    impl_name: Cow::Borrowed("duniter-gdev"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 1000,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
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
/// The `TransactionExtension` to the basic transaction logic.
pub type TxExtension = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    pallet_oneshot_account::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

pub type TechnicalCommitteeInstance = Instance2;

pub struct BaseCallFilter;
impl Contains<RuntimeCall> for BaseCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        // not allowed to run session calls directly
        // not allowed to burn currency
        !matches!(
            call,
            RuntimeCall::Session(_) | RuntimeCall::Balances(pallet_balances::Call::burn { .. })
        )
    }
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    codec::DecodeWithMemTracking,
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

// Dynamic parameters
pub type EpochDuration = pallet_duniter_test_parameters::BabeEpochDuration<Runtime>;
pub type CertPeriod = pallet_duniter_test_parameters::CertPeriod<Runtime>;
pub type MaxByIssuer = pallet_duniter_test_parameters::CertMaxByIssuer<Runtime>;
pub type MinReceivedCertToBeAbleToIssueCert =
    pallet_duniter_test_parameters::CertMinReceivedCertToIssueCert<Runtime>;
pub type ValidityPeriod = pallet_duniter_test_parameters::CertValidityPeriod<Runtime>;
pub type ConfirmPeriod = pallet_duniter_test_parameters::IdtyConfirmPeriod<Runtime>;
pub type IdtyCreationPeriod = pallet_duniter_test_parameters::IdtyCreationPeriod<Runtime>;
pub type MembershipPeriod = pallet_duniter_test_parameters::MembershipPeriod<Runtime>;
pub type MembershipRenewalPeriod = pallet_duniter_test_parameters::MembershipRenewalPeriod<Runtime>;
pub type UdCreationPeriod = pallet_duniter_test_parameters::UdCreationPeriod<Runtime>;
pub type UdReevalPeriod = pallet_duniter_test_parameters::UdReevalPeriod<Runtime>;
pub type WotFirstCertIssuableOn = pallet_duniter_test_parameters::WotFirstCertIssuableOn<Runtime>;
pub type WotMinCertForMembership = pallet_duniter_test_parameters::WotMinCertForMembership<Runtime>;
pub type WotMinCertForCreateIdtyRight =
    pallet_duniter_test_parameters::WotMinCertForCreateIdtyRight<Runtime>;
pub type SmithMaxByIssuer = pallet_duniter_test_parameters::SmithCertMaxByIssuer<Runtime>;
pub type SmithWotMinCertForMembership =
    pallet_duniter_test_parameters::SmithWotMinCertForMembership<Runtime>;
pub type SmithInactivityMaxDuration =
    pallet_duniter_test_parameters::SmithInactivityMaxDuration<Runtime>;

impl pallet_duniter_test_parameters::Config for Runtime {
    type BlockNumber = u32;
    type CertCount = u32;
    type PeriodCount = Balance;
    type SessionCount = u32;
}

// Create the runtime by composing the pallets that were previously configured.
construct_runtime!(
    pub enum Runtime    {
        // Basic stuff
        System: frame_system = 0,
        Account: pallet_duniter_account = 1,
        Scheduler: pallet_scheduler = 2,

        // Block creation
        Babe: pallet_babe = 3,
        Timestamp: pallet_timestamp = 4,

        // Test parameters
        Parameters: pallet_duniter_test_parameters = 5,

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

// All of our runtimes share most of their Runtime API implementations.
// We use a macro to implement this common part and add runtime-specific additional implementations.
common_runtime::pallets_config!();
common_runtime::declare_session_keys! {}
#[cfg(feature = "runtime-benchmarks")]
common_runtime::benchmarks_config!();
common_runtime::offchain_config! {}
common_runtime::runtime_apis! {}
