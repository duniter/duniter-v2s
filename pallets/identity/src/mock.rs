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
    parameter_types,
    traits::{Everything, GenesisBuild, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::{Header, TestSignature, UintAuthorityId},
    traits::{BlakeTwo256, IdentityLookup, IsMember},
};

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

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
    pub const IdtyCreationPeriod: u64 = 3;
    pub const MaxInactivityPeriod: u64 = 5;
    pub const RenewablePeriod: u64 = 3;
    pub const ValidationPeriod: u64 = 2;
}

pub struct IdtyNameValidatorTestImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorTestImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() < 16
    }
}

pub struct IsMemberTestImpl;
impl IsMember<u64> for IsMemberTestImpl {
    fn is_member(_: &u64) -> bool {
        true
    }
}

impl pallet_identity::Config for Test {
    type ConfirmPeriod = ConfirmPeriod;
    type Event = Event;
    type EnsureIdtyCallAllowed = ();
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type IdtyIndex = u64;
    type IdtyValidationOrigin = system::EnsureRoot<AccountId>;
    type IsMember = IsMemberTestImpl;
    type OnIdtyChange = ();
    type RemoveIdentityConsumers = ();
    type RevocationSigner = UintAuthorityId;
    type RevocationSignature = TestSignature;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_identity::GenesisConfig<Test>) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    gen_conf.assimilate_storage(&mut t).unwrap();

    frame_support::BasicExternalities::execute_with_storage(&mut t, || {
        // Some dedicated test account
        frame_system::Pallet::<Test>::inc_providers(&2);
        frame_system::Pallet::<Test>::inc_providers(&3);
    });

    sp_io::TestExternalities::new(t)
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Identity::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
    }
}
