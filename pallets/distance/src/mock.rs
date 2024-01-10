// Copyright 2023 Axiom-Team
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
use crate::{self as pallet_distance};
use core::marker::PhantomData;
use frame_support::{
    parameter_types,
    traits::{Everything, GenesisBuild},
};
use frame_system as system;
use pallet_balances::AccountData;
use pallet_session::ShouldEndSession;
use sp_core::{ConstU32, H256};
use sp_runtime::{
    impl_opaque_keys,
    key_types::DUMMY,
    testing::{Header, TestSignature, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, IsMember, OpaqueKeys},
    KeyTypeId, Perbill,
};

type Balance = u64;
type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type AccountId = u64;

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
        Authorship: pallet_authorship::{Pallet, Storage},
        AuthorityMembers: pallet_authority_members::{Pallet, Call, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Identity: pallet_identity::{Pallet, Call, Storage, Config<T>, Event<T>},
        Distance: pallet_distance::{Pallet, Call, Storage, Event<T>},
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
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[DUMMY];

    fn on_new_session<Ks: OpaqueKeys>(
        _changed: bool,
        _validators: &[(AccountId, Ks)],
        _queued_validators: &[(AccountId, Ks)],
    ) {
    }

    fn on_disabled(_validator_index: u32) {}

    fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}
}

const SESSION_LENGTH: u64 = 5;
pub struct TestShouldEndSession;
impl ShouldEndSession<u64> for TestShouldEndSession {
    fn should_end_session(now: u64) -> bool {
        now % SESSION_LENGTH == 0
    }
}

impl pallet_session::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
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

pub struct ConstantAuthor<T>(PhantomData<T>);

impl<T: From<u64>> frame_support::traits::FindAuthor<T> for ConstantAuthor<T> {
    fn find_author<'a, I>(_: I) -> Option<T>
    where
        I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
    {
        Some(1u64.into())
    }
}

impl pallet_authorship::Config for Test {
    type FindAuthor = ConstantAuthor<Self::AccountId>;
    type EventHandler = ();
}

pub struct TestIsSmithMember;
impl IsMember<u32> for TestIsSmithMember {
    fn is_member(member_id: &u32) -> bool {
        member_id % 3 == 0
    }
}

pub struct IdentityIndexOf<T: pallet_identity::Config>(PhantomData<T>);

impl<T: pallet_identity::Config> sp_runtime::traits::Convert<T::AccountId, Option<T::IdtyIndex>>
    for IdentityIndexOf<T>
{
    fn convert(account_id: T::AccountId) -> Option<T::IdtyIndex> {
        pallet_identity::Pallet::<T>::identity_index_of(account_id)
    }
}

impl pallet_authority_members::Config for Test {
    type IsMember = TestIsSmithMember;
    type MaxAuthorities = ConstU32<4>;
    type MemberId = u32;
    type MemberIdOf = IdentityIndexOf<Self>;
    type OnNewSession = ();
    type OnRemovedMember = ();
    type RemoveMemberOrigin = system::EnsureRoot<AccountId>;
    type RuntimeEvent = RuntimeEvent;
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
    pub const ChangeOwnerKeyPeriod: u64 = 10;
    pub const ConfirmPeriod: u64 = 2;
    pub const ValidationPeriod: u64 = 3;
    pub const AutorevocationPeriod: u64 = 5;
    pub const DeletionPeriod: u64 = 7;
    pub const IdtyCreationPeriod: u64 = 3;
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
    type ValidationPeriod = ValidationPeriod;
    type AutorevocationPeriod = AutorevocationPeriod;
    type DeletionPeriod = DeletionPeriod;
    type CheckIdtyCallAllowed = ();
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyData = ();
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type IdtyIndex = u32;
    type AccountLinker = ();
    type Signer = UintAuthorityId;
    type Signature = TestSignature;
    type OnIdtyChange = ();
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinAccessibleReferees: Perbill = Perbill::from_percent(80);
}
impl pallet_distance::Config for Test {
    type Currency = Balances;
    type EvaluationPrice = frame_support::traits::ConstU64<1000>;
    type MaxRefereeDistance = frame_support::traits::ConstU32<5>;
    type MinAccessibleReferees = MinAccessibleReferees;
    type ResultExpiration = frame_support::traits::ConstU32<720>;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)] // ??? Clippy triggers dead code for new_test_ext while it is used during test benchmark
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pub const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];
    pallet_identity::GenesisConfig::<Test> {
        identities: (1..=4)
            .map(|i| pallet_identity::GenesisIdty {
                index: i as u32,
                name: pallet_identity::IdtyName::from(NAMES[i - 1]),
                value: pallet_identity::IdtyValue {
                    data: (),
                    next_creatable_identity_on: 0,
                    owner_key: i as u64,
                    old_owner_key: None,
                    next_scheduled: 0,
                    status: pallet_identity::IdtyStatus::Member,
                },
            })
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    sp_io::TestExternalities::new(t)
}
