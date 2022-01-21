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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod parameters;

pub use self::parameters::*;
pub use common_runtime::{
    constants::*, entities::*, handlers::*, AccountId, Address, Balance, BlockNumber,
    FullIdentificationOfImpl, Hash, Header, IdtyIndex, Index, Signature,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_duniter_test_parameters::Parameters as GenesisParameters;
pub use pallet_identity::{IdtyStatus, IdtyValue};
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as session_historical;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
pub use pallet_universal_dividend;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{KeyTypeId, Perbill, Permill};

use common_runtime::{IdtyNameValidatorImpl, OwnerKeyOfImpl};
use frame_system::EnsureRoot;
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys};
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
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{KeyOwnerProofSystem, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    StorageValue,
};

pub mod opaque {
    use super::*;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub grandpa: Grandpa,
            pub babe: Babe,
            pub im_online: ImOnline,
            pub authority_discovery: AuthorityDiscovery,
        }
    }
}

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("gdev"),
    impl_name: create_runtime_str!("gdev"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 100,
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
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
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

pub struct BaseCallFilter;
impl frame_support::traits::Contains<Call> for BaseCallFilter {
    fn contains(call: &Call) -> bool {
        !matches!(
            call,
            Call::Membership(
                pallet_membership::Call::claim_membership { .. }
                    | pallet_membership::Call::revoke_membership { .. }
            ) | Call::Session(_)
        )
    }
}

// Configure FRAME pallets to include in runtime.
common_runtime::pallets_config! {
    // Dynamic parameters
    pub type EpochDuration = pallet_duniter_test_parameters::BabeEpochDuration<Runtime>;
    pub type CertPeriod = pallet_duniter_test_parameters::CertPeriod<Runtime>;
    pub type MaxByIssuer = pallet_duniter_test_parameters::CertMaxByIssuer<Runtime>;
    pub type MinReceivedCertToBeAbleToIssueCert = pallet_duniter_test_parameters::CertMinReceivedCertToIssueCert<Runtime>;
    pub type CertRenewablePeriod = pallet_duniter_test_parameters::CertRenewablePeriod<Runtime>;
    pub type ValidityPeriod = pallet_duniter_test_parameters::CertValidityPeriod<Runtime>;
    pub type ConfirmPeriod = pallet_duniter_test_parameters::IdtyConfirmPeriod<Runtime>;
    pub type IdtyCreationPeriod = pallet_duniter_test_parameters::IdtyCreationPeriod<Runtime>;
    pub type MaxDisabledPeriod = pallet_duniter_test_parameters::IdtyMaxDisabledPeriod<Runtime>;
    pub type MembershipPeriod = pallet_duniter_test_parameters::MembershipPeriod<Runtime>;
    pub type RenewablePeriod = pallet_duniter_test_parameters::MembershipRenewablePeriod<Runtime>;
    pub type PendingMembershipPeriod = pallet_duniter_test_parameters::PendingMembershipPeriod<Runtime>;
    pub type UdCreationPeriod = pallet_duniter_test_parameters::UdCreationPeriod<Runtime>;
    pub type UdFirstReeval = pallet_duniter_test_parameters::UdFirstReeval<Runtime>;
    pub type UdReevalPeriod = pallet_duniter_test_parameters::UdReevalPeriod<Runtime>;
    pub type UdReevalPeriodInBlocks = pallet_duniter_test_parameters::UdReevalPeriodInBlocks<Runtime>;
    pub type WotFirstCertIssuableOn = pallet_duniter_test_parameters::WotFirstCertIssuableOn<Runtime>;
    pub type WotMinCertForMembership = pallet_duniter_test_parameters::WotMinCertForMembership<Runtime>;
    pub type WotMinCertForCreateIdtyRight = pallet_duniter_test_parameters::WotMinCertForCreateIdtyRight<Runtime>;
    pub type SmithCertPeriod = pallet_duniter_test_parameters::SmithCertPeriod<Runtime>;
    pub type SmithMaxByIssuer = pallet_duniter_test_parameters::SmithCertMaxByIssuer<Runtime>;
    pub type SmithMinReceivedCertToBeAbleToIssueCert = pallet_duniter_test_parameters::SmithCertMinReceivedCertToIssueCert<Runtime>;
    pub type SmithCertRenewablePeriod = pallet_duniter_test_parameters::SmithCertRenewablePeriod<Runtime>;
    pub type SmithValidityPeriod = pallet_duniter_test_parameters::SmithCertValidityPeriod<Runtime>;
    pub type SmithMembershipPeriod = pallet_duniter_test_parameters::SmithMembershipPeriod<Runtime>;
    pub type SmithPendingMembershipPeriod = pallet_duniter_test_parameters::SmithPendingMembershipPeriod<Runtime>;
    pub type SmithRenewablePeriod = pallet_duniter_test_parameters::SmithMembershipRenewablePeriod<Runtime>;
    pub type SmithsWotFirstCertIssuableOn = pallet_duniter_test_parameters::SmithsWotFirstCertIssuableOn<Runtime>;
    pub type SmithsWotMinCertForMembership = pallet_duniter_test_parameters::SmithsWotMinCertForMembership<Runtime>;

    impl pallet_duniter_test_parameters::Config for Runtime {
        type CertCount = u32;
        type PeriodCount = Balance;
    }
    impl pallet_sudo::Config for Runtime {
        type Event = Event;
        type Call = Call;
    }
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = common_runtime::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 1,

        // Block creation
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,

        // Test parameters
        Parameters: pallet_duniter_test_parameters::{Pallet, Config<T>, Storage} = 4,

        // Money management
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 32,

        // Consensus support
        AuthorityMembers: pallet_authority_members::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 11,
        Offences: pallet_offences::{Pallet, Storage, Event} = 12,
        Historical: session_historical::{Pallet} = 13,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 14,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event} = 15,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 16,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 17,

        // Governance stuff
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 20,

        // Cunning utilities
        Utility: pallet_utility::{Pallet, Call, Event} = 30,

        // Universal dividend
        UdAccountsStorage: pallet_ud_accounts_storage::{Pallet, Config<T>, Storage} = 40,
        UniversalDividend: pallet_universal_dividend::{Pallet, Call, Config<T>, Storage, Event<T>} = 41,

        // Web Of Trust
        Wot: pallet_duniter_wot::<Instance1>::{Pallet} = 50,
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>} = 51,
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 52,
        Cert: pallet_certification::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 53,

        // Smiths Sub-Wot
        SmithsSubWot: pallet_duniter_wot::<Instance2>::{Pallet} = 60,
        SmithsMembership: pallet_membership::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>} = 62,
        SmithsCert: pallet_certification::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>} = 63,

        // Multisig dispatch.
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 70,
    }
);

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
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
