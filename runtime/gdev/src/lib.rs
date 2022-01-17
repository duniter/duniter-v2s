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
    constants::*, entities::IdtyRight, AccountId, Address, Balance, BlockNumber, Hash, Header,
    IdtyIndex, IdtyNameValidatorImpl, Index, Signature,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_identity::{IdtyStatus, IdtyValue};
use pallet_transaction_payment::CurrencyAdapter;
pub use pallet_universal_dividend;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

use common_runtime::{
    authorizations::{AddStrongCertOrigin, DelStrongCertOrigin, EnsureIdtyCallAllowedImpl},
    handlers::{
        OnIdtyChangeHandler, OnNewStrongCertHandler, OnRemovedStrongCertHandler,
        OnRightKeyChangeHandler,
    },
};
use frame_system::EnsureRoot;
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor};
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
        !matches!(call, Call::Membership(_))
    }
}

// Configure FRAME pallets to include in runtime.
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
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 1,

        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 32,

        // Consensus support.
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event} = 10,

        // Governance stuff.
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 20,

        // Cunning utilities.
        Utility: pallet_utility::{Pallet, Call, Event} = 30,

        // Universal dividend.
        UdAccountsStorage: pallet_ud_accounts_storage::{Pallet, Config<T>, Storage} = 40,
        UniversalDividend: pallet_universal_dividend::{Pallet, Call, Config<T>, Storage, Event<T>} = 41,

        // Web Of Trust
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>} = 50,
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 51,
        StrongCert: pallet_certification::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 52,

        // Multisig dispatch.
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 60,
    }
);

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
common_runtime::runtime_apis! {
    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<sp_authority_discovery::AuthorityId> {
            unimplemented!()
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            unimplemented!()
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            unimplemented!()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            unimplemented!()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            unimplemented!()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            _authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            unimplemented!()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            _key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            unimplemented!()
        }
    }
}
