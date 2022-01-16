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

use crate::{self as pallet_membership};
use frame_support::{
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_membership::traits::IsOriginAllowedToUseIdty;
use sp_membership::OriginPermission;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type AccountId = u64;
type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Test>;
pub type IdtyId = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        DefaultMembership: pallet_membership::{Pallet, Call, Event<T>, Storage, Config<T>},
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
    type BlockNumber = BlockNumber;
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

pub struct IsOriginAllowedToUseIdtyImpl;
impl IsOriginAllowedToUseIdty<Origin, IdtyId> for IsOriginAllowedToUseIdtyImpl {
    fn is_origin_allowed_to_use_idty(o: &Origin, idty_id: &IdtyId) -> OriginPermission {
        match o.clone().into() {
            Ok(system::RawOrigin::Root) => OriginPermission::Root,
            Ok(system::RawOrigin::Signed(account_id)) if account_id == *idty_id => {
                OriginPermission::Allowed
            }
            _ => OriginPermission::Forbidden,
        }
    }
}

parameter_types! {
    pub const ExternalizeMembershipStorage: bool = false;
    pub const MembershipPeriod: BlockNumber = 5;
    pub const PendingMembershipPeriod: BlockNumber = 3;
    pub const RenewablePeriod: BlockNumber = 2;
    pub const RevocationPeriod: BlockNumber = 4;
}

impl pallet_membership::Config for Test {
    type IsIdtyAllowedToClaimMembership = ();
    type IsIdtyAllowedToRenewMembership = ();
    type IsIdtyAllowedToRequestMembership = ();
    type IsOriginAllowedToUseIdty = IsOriginAllowedToUseIdtyImpl;
    type Event = Event;
    type ExternalizeMembershipStorage = ExternalizeMembershipStorage;
    type IdtyId = IdtyId;
    type OnEvent = ();
    type MembershipExternalStorage = ();
    type MembershipPeriod = MembershipPeriod;
    type PendingMembershipPeriod = PendingMembershipPeriod;
    type RenewablePeriod = RenewablePeriod;
    type RevocationPeriod = RevocationPeriod;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_membership::GenesisConfig<Test>) -> sp_io::TestExternalities {
    GenesisConfig {
        system: SystemConfig::default(),
        default_membership: gen_conf,
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        DefaultMembership::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        DefaultMembership::on_initialize(System::block_number());
    }
}
