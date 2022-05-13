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

use super::*;
use crate::{self as pallet_universal_dividend};
use frame_support::{
    parameter_types,
    traits::{Everything, Get, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Balance = u64;
type BlockNumber = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        UniversalDividend: pallet_universal_dividend::{Pallet, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
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

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const MembersCount: u64 = 3;
    pub const SquareMoneyGrowthRate: Perbill = Perbill::from_percent(10);
    pub const UdCreationPeriod: BlockNumber = 2;
    pub const UdReevalPeriod: BlockNumber = 8;
}

pub struct FakeWot;
impl Get<Vec<u64>> for FakeWot {
    fn get() -> Vec<u64> {
        vec![1, 2, 3]
    }
}

impl pallet_universal_dividend::Config for Test {
    type BlockNumberIntoBalance = sp_runtime::traits::ConvertInto;
    type Currency = pallet_balances::Pallet<Test>;
    type Event = Event;
    type MembersCount = MembersCount;
    type MembersIds = FakeWot;
    type SquareMoneyGrowthRate = SquareMoneyGrowthRate;
    type UdCreationPeriod = UdCreationPeriod;
    type UdReevalPeriod = UdReevalPeriod;
    type UnitsPerUd = frame_support::traits::ConstU64<1_000>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(
    gen_conf: pallet_universal_dividend::GenesisConfig<Test>,
) -> sp_io::TestExternalities {
    GenesisConfig {
        system: SystemConfig::default(),
        balances: BalancesConfig::default(),
        universal_dividend: gen_conf,
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        UniversalDividend::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        UniversalDividend::on_initialize(System::block_number());
    }
}
