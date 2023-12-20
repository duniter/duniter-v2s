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

use super::*;
use crate::{self as pallet_universal_dividend};
use frame_support::{
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::{ConstU32, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::traits::ConvertInto;

pub const BLOCK_TIME: u64 = 6_000;

type Balance = u64;
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
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
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
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u32;
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

parameter_types! {
    pub const MinimumPeriod: u64 = 3_000;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = UniversalDividend;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 10;
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

parameter_types! {
    pub const MembersCount: u64 = 3;
    pub const SquareMoneyGrowthRate: Perbill = Perbill::from_percent(10);
    pub const UdCreationPeriod: u64 = 12_000;
    pub const UdReevalPeriod: u64 = 48_000;
}

pub struct TestMembersStorage;
impl frame_support::traits::StoredMap<u32, FirstEligibleUd> for TestMembersStorage {
    fn get(key: &u32) -> FirstEligibleUd {
        crate::TestMembers::<Test>::get(key)
    }
    fn try_mutate_exists<R, E: From<sp_runtime::DispatchError>>(
        key: &u32,
        f: impl FnOnce(&mut Option<FirstEligibleUd>) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut value = Some(crate::TestMembers::<Test>::get(key));
        let result = f(&mut value)?;
        if let Some(value) = value {
            crate::TestMembers::<Test>::insert(key, value)
        }
        Ok(result)
    }
}

impl pallet_universal_dividend::Config for Test {
    type MomentIntoBalance = sp_runtime::traits::ConvertInto;
    type Currency = pallet_balances::Pallet<Test>;
    type MaxPastReeval = frame_support::traits::ConstU32<2>;
    type MembersCount = MembersCount;
    type MembersStorage = TestMembersStorage;
    type RuntimeEvent = RuntimeEvent;
    type SquareMoneyGrowthRate = SquareMoneyGrowthRate;
    type UdCreationPeriod = UdCreationPeriod;
    type UdReevalPeriod = UdReevalPeriod;
    type UnitsPerUd = frame_support::traits::ConstU64<1_000>;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type AccountIdOf = ConvertInto;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(
    gen_conf: pallet_universal_dividend::GenesisConfig<Test>,
) -> sp_io::TestExternalities {
    GenesisConfig {
        system: SystemConfig::default(),
        balances: BalancesConfig {
            total_issuance: gen_conf.initial_monetary_mass,
        },
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
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        UniversalDividend::on_initialize(System::block_number());
        Timestamp::set_timestamp(System::block_number() * BLOCK_TIME);
    }
}
