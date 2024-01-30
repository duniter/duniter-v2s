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

// Note: most of this file is copy pasted from common pallet_config and other mocks

use super::*;
pub use crate::pallet as pallet_quota;
use frame_support::{
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::{Pair, H256};
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::Verify;
use sp_runtime::BuildStorage;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    MultiSignature, MultiSigner,
};

type BlockNumber = u64;
type Balance = u64;
type Block = frame_system::mocking::MockBlock<Test>;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

pub fn account(id: u8) -> AccountId {
    let pair = sp_core::sr25519::Pair::from_seed(&[id; 32]);
    MultiSigner::Sr25519(pair.public()).into_account()
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test{
        System: frame_system,
        Quota: pallet_quota,
        Balances: pallet_balances,
        Identity: pallet_identity,
    }
);

// QUOTA //
pub struct TreasuryAccountId;
impl frame_support::pallet_prelude::Get<AccountId> for TreasuryAccountId {
    fn get() -> AccountId {
        account(99)
    }
}
parameter_types! {
    pub const ReloadRate: u64 = 10;
    pub const MaxQuota: u64 = 1000;
}
impl Config for Test {
    type MaxQuota = MaxQuota;
    type RefundAccount = TreasuryAccountId;
    type ReloadRate = ReloadRate;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// SYSTEM //
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}
impl system::Config for Test {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
    type Block = Block;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockWeights = ();
    type DbWeight = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = u64;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type PalletInfo = PalletInfo;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeTask = ();
    type SS58Prefix = SS58Prefix;
    type SystemWeightInfo = ();
    type Version = ();
}

// BALANCES //
parameter_types! {
    pub const ExistentialDeposit: Balance = 1000;
    pub const MaxLocks: u32 = 50;
}
impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
    type MaxHolds = ConstU32<0>;
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeEvent = RuntimeEvent;
    type RuntimeFreezeReason = ();
    type RuntimeHoldReason = ();
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

// IDENTITY //
parameter_types! {
    pub const ChangeOwnerKeyPeriod: u64 = 10;
    pub const ConfirmPeriod: u64 = 2;
    pub const ValidationPeriod: u64 = 3;
    pub const AutorevocationPeriod: u64 = 5;
    pub const DeletionPeriod: u64 = 7;
    pub const IdtyCreationPeriod: u64 = 3;
}
pub struct IdtyNameValidatorTestImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorTestImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() < 16
    }
}
impl pallet_identity::Config for Test {
    type AccountLinker = ();
    type AutorevocationPeriod = AutorevocationPeriod;
    type ChangeOwnerKeyPeriod = ChangeOwnerKeyPeriod;
    type CheckIdtyCallAllowed = ();
    type ConfirmPeriod = ConfirmPeriod;
    type DeletionPeriod = DeletionPeriod;
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyData = ();
    type IdtyIndex = u64;
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type OnIdtyChange = ();
    type RuntimeEvent = RuntimeEvent;
    type Signature = Signature;
    type Signer = AccountPublic;
    type ValidationPeriod = ValidationPeriod;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_quota::GenesisConfig<Test>) -> sp_io::TestExternalities {
    RuntimeGenesisConfig {
        system: SystemConfig::default(),
        balances: BalancesConfig::default(),
        quota: gen_conf,
        identity: IdentityConfig::default(),
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        <frame_system::Pallet<Test> as OnFinalize<BlockNumber>>::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        <frame_system::Pallet<Test> as OnInitialize<BlockNumber>>::on_initialize(
            System::block_number(),
        );
    }
}
