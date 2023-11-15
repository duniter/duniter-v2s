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
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    MultiSignature, MultiSigner,
};

type BlockNumber = u64;
type Balance = u64;
type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

pub fn account(id: u8) -> AccountId {
    let pair = sp_core::sr25519::Pair::from_seed(&[id; 32]);
    MultiSigner::Sr25519(pair.public()).into_account()
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Quota: pallet_quota::{Pallet, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Identity: pallet_identity::{Pallet, Call, Storage, Config<T>, Event<T>},
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
    type RuntimeEvent = RuntimeEvent;
    type ReloadRate = ReloadRate;
    type MaxQuota = MaxQuota;
    type RefundAccount = TreasuryAccountId;
    type WeightInfo = ();
}

// SYSTEM //
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}
impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// BALANCES //
parameter_types! {
    pub const ExistentialDeposit: Balance = 1000;
    pub const MaxLocks: u32 = 50;
}
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeEvent = RuntimeEvent;
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
}

// IDENTITY //
parameter_types! {
    pub const ChangeOwnerKeyPeriod: u64 = 10;
    pub const ConfirmPeriod: u64 = 2;
    pub const IdtyCreationPeriod: u64 = 3;
    pub const MaxInactivityPeriod: u64 = 5;
    pub const ValidationPeriod: u64 = 2;
}
pub struct IdtyNameValidatorTestImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorTestImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() < 16
    }
}
impl pallet_identity::Config for Test {
    type ChangeOwnerKeyPeriod = ChangeOwnerKeyPeriod;
    type ConfirmPeriod = ConfirmPeriod;
    type CheckIdtyCallAllowed = ();
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyData = ();
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type IdtyIndex = u64;
    type AccountLinker = ();
    type IdtyRemovalOtherReason = ();
    type Signer = AccountPublic;
    type Signature = Signature;
    type OnIdtyChange = ();
    type RemoveIdentityConsumers = ();
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_quota::GenesisConfig<Test>) -> sp_io::TestExternalities {
    GenesisConfig {
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
