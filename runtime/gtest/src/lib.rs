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
    FullIdentificationOfImpl, GetCurrentEpochIndex, Hash, Header, IdtyIndex, Index, Signature,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_identity::{IdtyStatus, IdtyValue};
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as session_historical;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
pub use pallet_universal_dividend;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{KeyTypeId, Perbill, Permill};

use common_runtime::IdtyNameValidatorImpl;
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
    traits::{EqualPrivilegeOnly, KeyOwnerProofSystem, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    StorageValue,
};

common_runtime::declare_session_keys! {}

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("gdem"),
    impl_name: create_runtime_str!("gdem"),
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
            Call::System(
                frame_system::Call::remark { .. } | frame_system::Call::remark_with_event { .. }
            ) | Call::Membership(
                pallet_membership::Call::claim_membership { .. }
                    | pallet_membership::Call::revoke_membership { .. }
            ) | Call::Session(_)
                | Call::SmithsMembership(pallet_membership::Call::claim_membership { .. })
        )
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
    Any = 0,
    TransferOnly = 1,
    CancelProxy = 2,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}
impl frame_support::traits::InstanceFilter<Call> for ProxyType {
    fn filter(&self, c: &Call) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::TransferOnly => {
                matches!(c, Call::Balances(..) | Call::UniversalDividend(..))
            }
            ProxyType::CancelProxy => {
                matches!(
                    c,
                    Call::Proxy(pallet_proxy::Call::reject_announcement { .. })
                )
            }
        }
    }
}

common_runtime::pallets_config! {
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
        Account: pallet_duniter_account::{Pallet, Storage, Config<T>, Event<T>} = 1,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 2,

        // Block creation
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 3,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 4,

        // Money management
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 32,

        // Consensus support.
        AuthorityMembers: pallet_authority_members::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 11,
        Offences: pallet_offences::{Pallet, Storage, Event} = 12,
        Historical: session_historical::{Pallet} = 13,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 14,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event} = 15,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 16,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 17,

        // Governance stuff.
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 20,

        // Universal dividend
        UdAccountsStorage: pallet_ud_accounts_storage::{Pallet, Config<T>, Storage} = 30,
        UniversalDividend: pallet_universal_dividend::{Pallet, Call, Config<T>, Storage, Event<T>} = 31,

        // Web Of Trust
        Wot: pallet_duniter_wot::<Instance1>::{Pallet} = 40,
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>} = 41,
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 42,
        Cert: pallet_certification::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 43,

        // Smiths Sub-Wot
        SmithsSubWot: pallet_duniter_wot::<Instance2>::{Pallet} = 50,
        SmithsMembership: pallet_membership::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>} = 52,
        SmithsCert: pallet_certification::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>} = 53,
        SmithsCollective: pallet_collective::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>, Origin<T>} = 54,

        // Utilities
        AtomicSwap: pallet_atomic_swap::{Pallet, Call, Storage, Event<T>} = 60,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 61,
        ProvideRandomness: pallet_provide_randomness::{Pallet, Call, Storage, Event} = 62,
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 63,
        Utility: pallet_utility::{Pallet, Call, Event} = 64,
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
