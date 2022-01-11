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
use crate::{self as pallet_identity};
use frame_support::{
    codec::{Decode, Encode},
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
    RuntimeDebug,
};
use frame_system as system;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    RuntimeDebug,
    Deserialize,
    Serialize,
    TypeInfo,
)]
pub enum IdtyRight {
    Right1,
    Right2,
}
impl Default for IdtyRight {
    fn default() -> Self {
        IdtyRight::Right1
    }
}
impl pallet_identity::traits::IdtyRight for IdtyRight {
    fn allow_owner_key(self) -> bool {
        self == Self::Right1
    }
    fn create_idty_right() -> Self {
        Self::Right1
    }
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Identity: pallet_identity::{Pallet, Call, Storage, Config<T>, Event<T>},
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
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ConfirmPeriod: u64 = 2;
    pub const MaxInactivityPeriod: u64 = 5;
    pub const MaxNoRightPeriod: u64 = 4;
    pub const RenewablePeriod: u64 = 3;
    pub const ValidationPeriod: u64 = 2;
}

pub struct IdtyNameValidatorTestImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorTestImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() == 1
    }
}

impl pallet_identity::Config for Test {
    type ConfirmPeriod = ConfirmPeriod;
    type Event = Event;
    type AddRightOrigin = system::EnsureRoot<AccountId>;
    type DelRightOrigin = system::EnsureRoot<AccountId>;
    type EnsureIdtyCallAllowed = ();
    type IdtyData = ();
    type IdtyDataProvider = ();
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type IdtyIndex = u64;
    type IdtyValidationOrigin = system::EnsureRoot<AccountId>;
    type IdtyRight = IdtyRight;
    type OnIdtyChange = ();
    type OnRightKeyChange = ();
    type MaxInactivityPeriod = MaxInactivityPeriod;
    type MaxNoRightPeriod = MaxNoRightPeriod;
    type RenewablePeriod = RenewablePeriod;
    type ValidationPeriod = ValidationPeriod;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_identity::GenesisConfig<Test>) -> sp_io::TestExternalities {
    GenesisConfig {
        system: SystemConfig::default(),
        identity: gen_conf,
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Identity::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
    }
}
