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
use crate::{self as pallet_authority_members};
use frame_support::{pallet_prelude::*, parameter_types, traits::Everything};
use frame_system as system;
use pallet_offences::{traits::OnOffenceHandler, SlashStrategy};
use pallet_session::ShouldEndSession;
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::{
    impl_opaque_keys,
    testing::UintAuthorityId,
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, IsMember, OpaqueKeys},
    BuildStorage, KeyTypeId,
};
use sp_staking::offence::OffenceDetails;
use sp_state_machine::BasicExternalities;

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

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
    pub enum Test
    {
        System: frame_system,
        Session: pallet_session,
        AuthorityMembers: pallet_authority_members,
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
    type SS58Prefix = SS58Prefix;
    type SingleBlockMigrations = ();
    type SystemWeightInfo = ();
    type Version = ();
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
    type Keys = MockSessionKeys;
    type NextSessionRotation = ();
    type RuntimeEvent = RuntimeEvent;
    type SessionHandler = TestSessionHandler;
    type SessionManager = AuthorityMembers;
    type ShouldEndSession = TestShouldEndSession;
    type ValidatorId = u64;
    type ValidatorIdOf = ConvertInto;
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
    type IsMember = TestIsSmithMember;
    type MaxAuthorities = ConstU32<4>;
    type MemberId = u64;
    type MemberIdOf = ConvertInto;
    type OnIncomingMember = ();
    type OnNewSession = ();
    type OnOutgoingMember = ();
    type RemoveMemberOrigin = system::EnsureRoot<u64>;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(initial_authorities_len: u64) -> sp_io::TestExternalities {
    let initial_authorities = (1..=initial_authorities_len)
        .map(|i| (i * 3, (i * 3, true)))
        .collect();
    let keys: Vec<_> = (1..=initial_authorities_len)
        .map(|i| (i * 3, i * 3, UintAuthorityId(i * 3).into()))
        .collect();

    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
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

pub(crate) fn on_offence(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_strategy: SlashStrategy,
) {
    AuthorityMembers::on_offence(offenders, slash_strategy, 0);
}
