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
    constants::*,
    entities::{IdtyData, IdtyRight, ValidatorFullIdentification},
    AccountId, Address, Balance, BlockNumber, FullIdentificationOfImpl, Hash, Header, IdtyIndex,
    IdtyNameValidatorImpl, Index, Signature,
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

use common_runtime::{
    authorizations::{AddStrongCertOrigin, DelStrongCertOrigin, EnsureIdtyCallAllowedImpl},
    handlers::{
        OnIdtyChangeHandler, OnNewStrongCertHandler, OnRemovedStrongCertHandler,
        OnRightKeyChangeHandler,
    },
    providers::IdtyDataProvider,
    SessionManagerImpl,
};
use frame_system::EnsureRoot;
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
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
    spec_name: create_runtime_str!("g1"),
    impl_name: create_runtime_str!("g1"),
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

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 42;
    pub const UncleGenerations: u32 = 0;
}

// Configure FRAME pallets to include in runtime.
common_runtime::pallets_config! {
    impl pallet_authority_discovery::Config for Runtime {
        type MaxAuthorities = MaxAuthorities;
    }
    impl pallet_authorship::Config for Runtime {
        type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
        type UncleGenerations = UncleGenerations;
        type FilterUncle = ();
        type EventHandler = ImOnline;
    }
    impl pallet_babe::Config for Runtime {
        type EpochDuration = EpochDuration;
        type ExpectedBlockTime = ExpectedBlockTime;

        // session module is the trigger
        type EpochChangeTrigger = pallet_babe::ExternalTrigger;

        type DisabledValidators = Session;

        type KeyOwnerProofSystem = Historical;

        type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
            KeyTypeId,
            pallet_babe::AuthorityId,
        )>>::Proof;

        type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
            KeyTypeId,
            pallet_babe::AuthorityId,
        )>>::IdentificationTuple;

        type HandleEquivocation =
            pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

        type WeightInfo = ();

        type MaxAuthorities = MaxAuthorities;
    }
    parameter_types! {
        pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
        pub const MaxKeys: u32 = 10_000;
        pub const MaxPeerInHeartbeats: u32 = 10_000;
        pub const MaxPeerDataEncodingSize: u32 = 1_000;
    }
    impl pallet_im_online::Config for Runtime {
        type AuthorityId = ImOnlineId;
        type Event = Event;
        type ValidatorSet = Historical;
        type NextSessionRotation = Babe;
        type ReportUnresponsiveness = Offences;
        type UnsignedPriority = ImOnlineUnsignedPriority;
        type WeightInfo = ();
        type MaxKeys = MaxKeys;
        type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
        type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
    }
    impl pallet_offences::Config for Runtime {
        type Event = Event;
        type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
        type OnOffenceHandler = ();
    }
    impl pallet_session::Config for Runtime {
        type Event = Event;
        type ValidatorId = AccountId;
        type ValidatorIdOf = sp_runtime::traits::ConvertInto;
        type ShouldEndSession = Babe;
        type NextSessionRotation = Babe;
        type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, SessionManagerImpl>;
        type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
        type Keys = opaque::SessionKeys;
        type WeightInfo = ();
    }
    impl pallet_session::historical::Config for Runtime {
        type FullIdentification = ValidatorFullIdentification;
        type FullIdentificationOf = FullIdentificationOfImpl;
    }
    impl pallet_timestamp::Config for Runtime {
        type Moment = u64;
        type OnTimestampSet = Babe;
        type MinimumPeriod = MinimumPeriod;
        type WeightInfo = ();
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

        // Babe must be before session.
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 2,

        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,

        // Money management
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 32,

        // Consensus support.
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 10,
        Offences: pallet_offences::{Pallet, Storage, Event} = 11,
        Historical: session_historical::{Pallet} = 12,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 13,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event} = 14,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 15,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 16,

        // Governance stuff.
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 20,

        // Cunning utilities.
        Utility: pallet_utility::{Pallet, Call, Event} = 30,

        // Universal dividend.
        UdAccountsStorage: pallet_ud_accounts_storage::{Pallet, Config<T>, Storage} = 40,
        UniversalDividend: pallet_universal_dividend::{Pallet, Call, Config<T>, Storage, Event<T>} = 41,

        // Web Of Trust
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>} = 50,
        StrongCert: pallet_certification::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>} = 51,

        // Multisig dispatch.
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 60,
    }
);

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
common_runtime::runtime_apis! {
    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<sp_authority_discovery::AuthorityId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: BABE_GENESIS_EPOCH_CONFIG.c,
                genesis_authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }
}
