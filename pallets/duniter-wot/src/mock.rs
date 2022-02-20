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
use crate::{self as pallet_duniter_wot};
use frame_support::{parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use std::collections::BTreeMap;

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
        DuniterWot: pallet_duniter_wot::<Instance1>::{Pallet},
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>},
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>},
        Cert: pallet_certification::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>},
    }
);

// System
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

// DuniterWot
parameter_types! {
    pub const MinCertForMembership: u32 = 2;
    pub const MinCertForCreateIdtyRigh: u32 = 4;
    pub const FirstIssuableOn: u64 = 2;
}

impl pallet_duniter_wot::Config<Instance1> for Test {
    type IsSubWot = frame_support::traits::ConstBool<false>;
    type MinCertForMembership = MinCertForMembership;
    type MinCertForCreateIdtyRight = MinCertForCreateIdtyRigh;
    type FirstIssuableOn = FirstIssuableOn;
}

// Identity
parameter_types! {
    pub const ConfirmPeriod: u64 = 2;
    pub const IdtyCreationPeriod: u64 = 3;
    pub const MaxDisabledPeriod: u64 = 4;
    pub const ValidationPeriod: u64 = 2;
}

pub struct IdtyNameValidatorTestImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorTestImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() < 16
    }
}

impl pallet_identity::Config for Test {
    type ConfirmPeriod = ConfirmPeriod;
    type Event = Event;
    type EnsureIdtyCallAllowed = DuniterWot;
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type IdtyIndex = IdtyIndex;
    type IdtyValidationOrigin = system::EnsureRoot<AccountId>;
    type IsMember = Membership;
    type OnIdtyChange = DuniterWot;
    type MaxDisabledPeriod = MaxDisabledPeriod;
    type RemoveIdentityConsumers = ();
}

// Membership
parameter_types! {
    pub const MembershipPeriod: u64 = 5;
    pub const PendingMembershipPeriod: u64 = 3;
    pub const RenewablePeriod: u64 = 2;
    pub const RevocationPeriod: u64 = 4;
}

impl pallet_membership::Config<Instance1> for Test {
    type IsIdtyAllowedToRenewMembership = DuniterWot;
    type IsIdtyAllowedToRequestMembership = DuniterWot;
    type Event = Event;
    type IdtyId = IdtyIndex;
    type IdtyIdOf = Identity;
    type MembershipPeriod = MembershipPeriod;
    type MetaData = crate::MembershipMetaData<u64>;
    type OnEvent = DuniterWot;
    type PendingMembershipPeriod = PendingMembershipPeriod;
    type RenewablePeriod = RenewablePeriod;
    type RevocationPeriod = RevocationPeriod;
}

// Cert
parameter_types! {
    pub const MaxByIssuer: u8 = 8;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 2;
    pub const CertRenewablePeriod: u64 = 4;
    pub const CertPeriod: u64 = 2;
    pub const ValidityPeriod: u64 = 10;
}

impl pallet_certification::Config<Instance1> for Test {
    type CertPeriod = CertPeriod;
    type Event = Event;
    type IdtyIndex = IdtyIndex;
    type IdtyIndexOf = Identity;
    type IsCertAllowed = DuniterWot;
    type MaxByIssuer = MaxByIssuer;
    type MinReceivedCertToBeAbleToIssueCert = MinReceivedCertToBeAbleToIssueCert;
    type OnNewcert = DuniterWot;
    type OnRemovedCert = DuniterWot;
    type CertRenewablePeriod = CertRenewablePeriod;
    type ValidityPeriod = ValidityPeriod;
}

pub const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(initial_identities_len: usize) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_identity::GenesisConfig::<Test> {
        identities: (1..=initial_identities_len)
            .map(|i| pallet_identity::GenesisIdty {
                index: i as u32,
                name: pallet_identity::IdtyName::from(NAMES[i - 1]),
                value: pallet_identity::IdtyValue {
                    next_creatable_identity_on: 0,
                    owner_key: i as u64,
                    removable_on: 0,
                    status: pallet_identity::IdtyStatus::Validated,
                },
            })
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_membership::GenesisConfig::<Test, Instance1> {
        memberships: (1..=initial_identities_len)
            .map(|i| {
                (
                    i as u32,
                    sp_membership::MembershipData {
                        expire_on: MembershipPeriod::get(),
                        renewable_on: RenewablePeriod::get(),
                    },
                )
            })
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_certification::GenesisConfig::<Test, Instance1> {
        certs_by_issuer: clique_wot(initial_identities_len, ValidityPeriod::get()),
        apply_cert_period_at_genesis: true,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    frame_support::BasicExternalities::execute_with_storage(&mut t, || {
        // manually increment genesis identities sufficient counter
        // In real world, this should be handle manually by genesis creator
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
        DuniterWot::on_finalize(System::block_number());
        Identity::on_finalize(System::block_number());
        Membership::on_finalize(System::block_number());
        Cert::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
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
) -> BTreeMap<IdtyIndex, BTreeMap<IdtyIndex, u64>> {
    let mut certs_by_issuer = BTreeMap::new();
    for i in 1..=initial_identities_len {
        certs_by_issuer.insert(
            i as IdtyIndex,
            (1..=initial_identities_len)
                .filter_map(|j| {
                    if i != j {
                        Some((j as IdtyIndex, cert_validity_period))
                    } else {
                        None
                    }
                })
                .collect(),
        );
    }
    certs_by_issuer
}
