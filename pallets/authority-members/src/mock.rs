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
use crate::{self as pallet_authority_members};
use frame_support::{
    pallet_prelude::*,
    parameter_types,
    traits::{Everything, GenesisBuild},
    BasicExternalities,
};
use frame_system as system;
use pallet_session::ShouldEndSession;
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::{
    impl_opaque_keys,
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, IsMember, OpaqueKeys},
    KeyTypeId,
};

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

impl From<UintAuthorityId> for MockSessionKeys {
    fn from(dummy: UintAuthorityId) -> Self {
        Self { dummy }
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
        Session: pallet_session::{Pallet, Call, Storage, Config<T>, Event},
        AuthorityMembers: pallet_authority_members::{Pallet, Call, Storage, Config<T>, Event<T>},
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

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<u64> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[DUMMY];

    fn on_new_session<Ks: OpaqueKeys>(
        _changed: bool,
        _validators: &[(u64, Ks)],
        _queued_validators: &[(u64, Ks)],
    ) {
    }

    fn on_disabled(_validator_index: u32) {}

    fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(u64, Ks)]) {}
}

const SESSION_LENGTH: u64 = 5;
pub struct TestShouldEndSession;
impl ShouldEndSession<u64> for TestShouldEndSession {
    fn should_end_session(now: u64) -> bool {
        now % SESSION_LENGTH == 0
    }
}

impl pallet_session::Config for Test {
    type Event = Event;
    type ValidatorId = u64;
    type ValidatorIdOf = ConvertInto;
    type ShouldEndSession = TestShouldEndSession;
    type NextSessionRotation = ();
    type SessionManager = AuthorityMembers;
    type SessionHandler = TestSessionHandler;
    type Keys = MockSessionKeys;
    type WeightInfo = ();
}

pub struct FullIdentificationOfImpl;
impl sp_runtime::traits::Convert<AccountId, Option<()>> for FullIdentificationOfImpl {
    fn convert(_: AccountId) -> Option<()> {
        Some(())
    }
}
impl pallet_session::historical::Config for Test {
    type FullIdentification = ();
    type FullIdentificationOf = FullIdentificationOfImpl;
}

pub struct TestIsSmithMember;
impl IsMember<u64> for TestIsSmithMember {
    fn is_member(member_id: &u64) -> bool {
        member_id % 3 == 0
    }
}

impl pallet_authority_members::Config for Test {
    type Event = Event;
    type KeysWrapper = MockSessionKeys;
    type IsMember = TestIsSmithMember;
    type MaxAuthorities = ConstU32<4>;
    type MaxKeysLife = ConstU32<5>;
    type MaxOfflineSessions = ConstU32<2>;
    type MemberId = u64;
    type MemberIdOf = ConvertInto;
    type OnNewSession = ();
    type OnRemovedMember = ();
    type RemoveMemberOrigin = system::EnsureRoot<u64>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(initial_authorities_len: u64) -> sp_io::TestExternalities {
    let initial_authorities = (1..=initial_authorities_len)
        .map(|i| (i * 3, (i * 3, true)))
        .collect();
    let keys: Vec<_> = (1..=initial_authorities_len)
        .map(|i| (i * 3, i * 3, UintAuthorityId(i * 3).into()))
        .collect();

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    BasicExternalities::execute_with_storage(&mut t, || {
        for (ref k, ..) in &keys {
            frame_system::Pallet::<Test>::inc_providers(k);
        }
        // Some dedicated test account
        frame_system::Pallet::<Test>::inc_providers(&12);
        frame_system::Pallet::<Test>::inc_providers(&15);
    });
    pallet_authority_members::GenesisConfig::<Test> {
        initial_authorities,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_session::GenesisConfig::<Test> { keys }
        .assimilate_storage(&mut t)
        .unwrap();
    sp_io::TestExternalities::new(t)
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Session::on_finalize(System::block_number());
        AuthorityMembers::on_initialize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        AuthorityMembers::on_initialize(System::block_number());
        Session::on_initialize(System::block_number());
    }
}
