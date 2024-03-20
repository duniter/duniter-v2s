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

use crate::{self as pallet_membership};
use frame_support::{
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, ConvertInto, IdentityLookup},
    BuildStorage,
};

type AccountId = u64;
type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Test>;
pub type IdtyId = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test{
        System: frame_system,
        Membership: pallet_membership,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type AccountData = ();
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

parameter_types! {
    pub const MembershipPeriod: BlockNumber = 5;
    pub const MembershipRenewalPeriod: BlockNumber = 2;
}

impl pallet_membership::Config for Test {
    type AccountIdOf = ConvertInto;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkSetupHandler = ();
    type CheckMembershipOpAllowed = ();
    type IdtyId = IdtyId;
    type IdtyIdOf = ConvertInto;
    type MembershipPeriod = MembershipPeriod;
    type MembershipRenewalPeriod = MembershipRenewalPeriod;
    type OnNewMembership = ();
    type OnRemoveMembership = ();
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_membership::GenesisConfig<Test>) -> sp_io::TestExternalities {
    RuntimeGenesisConfig {
        system: SystemConfig::default(),
        membership: gen_conf,
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Membership::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Membership::on_initialize(System::block_number());
    }
}
