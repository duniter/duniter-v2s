// Copyright 2021-2023 Axiom-Team
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

#![cfg(test)]

use crate::{self as pallet_smith_members};
use frame_support::pallet_prelude::Hooks;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
    weights::{constants::RocksDbWeight, Weight},
};
use sp_core::H256;
use sp_runtime::traits::{ConvertInto, IsMember};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, Perbill,
};

parameter_types! {
    pub static OnOffencePerbill: Vec<Perbill> = Default::default();
    pub static OffenceWeight: Weight = Default::default();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

frame_support::construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Smith: pallet_smith_members,
    }
);

impl frame_system::Config for Runtime {
    type AccountData = ();
    type AccountId = u64;
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockWeights = ();
    type DbWeight = RocksDbWeight;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = ConstU32<16>;
    type MultiBlockMigrator = ();
    type Nonce = u64;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type PalletInfo = PalletInfo;
    type PostInherents = ();
    type PostTransactions = ();
    type PreInherents = ();
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeTask = ();
    type SS58Prefix = ();
    type SingleBlockMigrations = ();
    type SystemWeightInfo = ();
    type Version = ();
}

pub struct EveryoneExceptIdZero;
impl IsMember<u64> for EveryoneExceptIdZero {
    fn is_member(member_id: &u64) -> bool {
        member_id != &0 && member_id != &10
    }
}

impl pallet_smith_members::Config for Runtime {
    type IdtyIdOf = ConvertInto;
    type IdtyIdOfAuthorityId = ConvertInto;
    type IdtyIndex = u64;
    type IsWoTMember = EveryoneExceptIdZero;
    type MaxByIssuer = ConstU32<3>;
    type MemberId = u64;
    type MinCertForMembership = ConstU32<2>;
    type OnSmithDelete = ();
    type OwnerKeyOf = ConvertInto;
    type RuntimeEvent = RuntimeEvent;
    type SmithInactivityMaxDuration = ConstU32<5>;
    type WeightInfo = ();
}

pub fn new_test_ext(
    genesis_config: crate::pallet::GenesisConfig<Runtime>,
) -> sp_io::TestExternalities {
    RuntimeGenesisConfig {
        system: SystemConfig::default(),
        smith: genesis_config,
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Smith::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Smith::on_initialize(System::block_number());
    }
}
