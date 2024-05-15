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
use crate::{self as pallet_duniter_wot};
use frame_support::{parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::{TestSignature, UintAuthorityId},
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use sp_state_machine::BasicExternalities;
use std::collections::BTreeMap;

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        DuniterWot: pallet_duniter_wot,
        Identity: pallet_identity,
        Membership: pallet_membership,
        Cert: pallet_certification,
    }
);

// System
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

// DuniterWot
parameter_types! {
    pub const MinCertForMembership: u32 = 2;
    pub const MinCertForCreateIdtyRight: u32 = 4;
    pub const FirstIssuableOn: u64 = 2;
}

impl pallet_duniter_wot::Config for Test {
    type FirstIssuableOn = FirstIssuableOn;
    type MinCertForCreateIdtyRight = MinCertForCreateIdtyRight;
    type MinCertForMembership = MinCertForMembership;
}

// Identity
parameter_types! {
    pub const ChangeOwnerKeyPeriod: u64 = 10;
    pub const ConfirmPeriod: u64 = 2;
    pub const ValidationPeriod: u64 = 5;
    pub const AutorevocationPeriod: u64 = 6;
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
    type AccountLinker = ();
    type AutorevocationPeriod = AutorevocationPeriod;
    type ChangeOwnerKeyPeriod = ChangeOwnerKeyPeriod;
    type CheckIdtyCallAllowed = DuniterWot;
    type ConfirmPeriod = ConfirmPeriod;
    type DeletionPeriod = DeletionPeriod;
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyData = ();
    type IdtyIndex = IdtyIndex;
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type OnNewIdty = DuniterWot;
    type OnRemoveIdty = DuniterWot;
    type RuntimeEvent = RuntimeEvent;
    type Signature = TestSignature;
    type Signer = UintAuthorityId;
    type ValidationPeriod = ValidationPeriod;
    type WeightInfo = ();
}

// Membership
parameter_types! {
    pub const MembershipPeriod: u64 = 8;
    pub const MembershipRenewalPeriod: u64 = 2;
}

impl pallet_membership::Config for Test {
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkSetupHandler = ();
    type CheckMembershipOpAllowed = DuniterWot;
    type IdtyAttr = Identity;
    type IdtyId = IdtyIndex;
    type MembershipPeriod = MembershipPeriod;
    type MembershipRenewalPeriod = MembershipRenewalPeriod;
    type OnNewMembership = DuniterWot;
    type OnRemoveMembership = DuniterWot;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Cert
parameter_types! {
    pub const MaxByIssuer: u8 = 8;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 2;
    pub const CertPeriod: u64 = 2;
    pub const ValidityPeriod: u64 = 20;
}

impl pallet_certification::Config for Test {
    type CertPeriod = CertPeriod;
    type CheckCertAllowed = DuniterWot;
    type IdtyAttr = Identity;
    type IdtyIndex = IdtyIndex;
    type MaxByIssuer = MaxByIssuer;
    type MinReceivedCertToBeAbleToIssueCert = MinReceivedCertToBeAbleToIssueCert;
    type OnNewcert = DuniterWot;
    type OnRemovedCert = DuniterWot;
    type RuntimeEvent = RuntimeEvent;
    type ValidityPeriod = ValidityPeriod;
    type WeightInfo = ();
}

pub const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(initial_identities_len: usize) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_identity::GenesisConfig::<Test> {
        identities: (1..=initial_identities_len)
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

    pallet_membership::GenesisConfig::<Test> {
        memberships: (1..=initial_identities_len)
            .map(|i| {
                (
                    i as u32,
                    sp_membership::MembershipData {
                        expire_on: MembershipPeriod::get(),
                    },
                )
            })
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_certification::GenesisConfig::<Test> {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: clique_wot(initial_identities_len, ValidityPeriod::get()),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    BasicExternalities::execute_with_storage(&mut t, || {
        // manually increment genesis identities sufficient counter
        // In real world, this is done by pallet-identity
        for i in 1..=initial_identities_len {
            frame_system::Pallet::<Test>::inc_sufficients(&(i as u64));
        }
        // Some dedicated test account
        frame_system::Pallet::<Test>::inc_providers(&(initial_identities_len as u64));
        frame_system::Pallet::<Test>::inc_providers(&(initial_identities_len as u64 + 1));
    });

    sp_io::TestExternalities::new(t)
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        // finalize previous block
        DuniterWot::on_finalize(System::block_number());
        Identity::on_finalize(System::block_number());
        Membership::on_finalize(System::block_number());
        Cert::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        // reset events and change block number
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        // initialize next block
        System::on_initialize(System::block_number());
        DuniterWot::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
        Membership::on_initialize(System::block_number());
        Cert::on_initialize(System::block_number());
    }
}

fn clique_wot(
    initial_identities_len: usize,
    cert_validity_period: u64,
) -> BTreeMap<IdtyIndex, BTreeMap<IdtyIndex, Option<u64>>> {
    let mut certs_by_issuer = BTreeMap::new();
    for i in 1..=initial_identities_len {
        certs_by_issuer.insert(
            i as IdtyIndex,
            (1..=initial_identities_len)
                .filter_map(|j| {
                    if i != j {
                        Some((j as IdtyIndex, Some(cert_validity_period)))
                    } else {
                        None
                    }
                })
                .collect(),
        );
    }
    certs_by_issuer
}
