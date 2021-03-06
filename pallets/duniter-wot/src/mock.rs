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
    testing::{Header, TestSignature, UintAuthorityId},
    traits::{BlakeTwo256, IdentityLookup},
};
use std::collections::BTreeMap;

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

pub struct IdentityIndexOf<T: pallet_identity::Config>(PhantomData<T>);
impl<T: pallet_identity::Config> sp_runtime::traits::Convert<T::AccountId, Option<T::IdtyIndex>>
    for IdentityIndexOf<T>
{
    fn convert(account_id: T::AccountId) -> Option<T::IdtyIndex> {
        pallet_identity::Pallet::<T>::identity_index_of(account_id)
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
        DuniterWot: pallet_duniter_wot::<Instance1>::{Pallet},
        Identity: pallet_identity::{Pallet, Call, Config<T>, Storage, Event<T>},
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>},
        Cert: pallet_certification::<Instance1>::{Pallet, Call, Config<T>, Storage, Event<T>},
        SmithsSubWot: pallet_duniter_wot::<Instance2>::{Pallet},
        SmithsMembership: pallet_membership::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>},
        SmithsCert: pallet_certification::<Instance2>::{Pallet, Call, Config<T>, Storage, Event<T>},
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
    pub const ChangeOwnerKeyPeriod: u64 = 10;
    pub const ConfirmPeriod: u64 = 2;
    pub const IdtyCreationPeriod: u64 = 3;
    pub const ValidationPeriod: u64 = 2;
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
    type Event = Event;
    type EnsureIdtyCallAllowed = DuniterWot;
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyData = ();
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type IdtyIndex = IdtyIndex;
    type NewOwnerKeySigner = UintAuthorityId;
    type NewOwnerKeySignature = TestSignature;
    type OnIdtyChange = DuniterWot;
    type RemoveIdentityConsumers = ();
    type RevocationSigner = UintAuthorityId;
    type RevocationSignature = TestSignature;
}

// Membership
parameter_types! {
    pub const MembershipPeriod: u64 = 8;
    pub const PendingMembershipPeriod: u64 = 3;
    pub const RevocationPeriod: u64 = 4;
}

impl pallet_membership::Config<Instance1> for Test {
    type IsIdtyAllowedToClaimMembership = DuniterWot;
    type IsIdtyAllowedToRenewMembership = DuniterWot;
    type IsIdtyAllowedToRequestMembership = DuniterWot;
    type Event = Event;
    type IdtyId = IdtyIndex;
    type IdtyIdOf = IdentityIndexOf<Self>;
    type MembershipPeriod = MembershipPeriod;
    type MetaData = crate::MembershipMetaData<u64>;
    type OnEvent = DuniterWot;
    type PendingMembershipPeriod = PendingMembershipPeriod;
    type RevocationPeriod = RevocationPeriod;
}

// Cert
parameter_types! {
    pub const MaxByIssuer: u8 = 8;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 2;
    pub const CertPeriod: u64 = 2;
    pub const ValidityPeriod: u64 = 20;
}

impl pallet_certification::Config<Instance1> for Test {
    type CertPeriod = CertPeriod;
    type Event = Event;
    type IdtyIndex = IdtyIndex;
    type OwnerKeyOf = Identity;
    type IsCertAllowed = DuniterWot;
    type MaxByIssuer = MaxByIssuer;
    type MinReceivedCertToBeAbleToIssueCert = MinReceivedCertToBeAbleToIssueCert;
    type OnNewcert = DuniterWot;
    type OnRemovedCert = DuniterWot;
    type ValidityPeriod = ValidityPeriod;
}

// SMITHS SUB-WOT //

parameter_types! {
    pub const SmithsMinCertForMembership: u32 = 2;
    pub const SmithsFirstIssuableOn: u64 = 2;
}

impl pallet_duniter_wot::Config<Instance2> for Test {
    type IsSubWot = frame_support::traits::ConstBool<true>;
    type MinCertForMembership = SmithsMinCertForMembership;
    type MinCertForCreateIdtyRight = frame_support::traits::ConstU32<0>;
    type FirstIssuableOn = SmithsFirstIssuableOn;
}

// SmithsMembership
parameter_types! {
    pub const SmithsMembershipPeriod: u64 = 20;
    pub const SmithsPendingMembershipPeriod: u64 = 3;
    pub const SmithsRevocationPeriod: u64 = 4;
}

impl pallet_membership::Config<Instance2> for Test {
    type IsIdtyAllowedToClaimMembership = SmithsSubWot;
    type IsIdtyAllowedToRenewMembership = SmithsSubWot;
    type IsIdtyAllowedToRequestMembership = SmithsSubWot;
    type Event = Event;
    type IdtyId = IdtyIndex;
    type IdtyIdOf = IdentityIndexOf<Self>;
    type MembershipPeriod = SmithsMembershipPeriod;
    type MetaData = crate::MembershipMetaData<u64>;
    type OnEvent = SmithsSubWot;
    type PendingMembershipPeriod = SmithsPendingMembershipPeriod;
    type RevocationPeriod = SmithsRevocationPeriod;
}

// SmithsCert
parameter_types! {
    pub const SmithsMaxByIssuer: u8 = 8;
    pub const SmithsMinReceivedCertToBeAbleToIssueCert: u32 = 2;
    pub const SmithsCertPeriod: u64 = 2;
    pub const SmithsValidityPeriod: u64 = 10;
}

impl pallet_certification::Config<Instance2> for Test {
    type CertPeriod = SmithsCertPeriod;
    type Event = Event;
    type IdtyIndex = IdtyIndex;
    type OwnerKeyOf = Identity;
    type IsCertAllowed = SmithsSubWot;
    type MaxByIssuer = SmithsMaxByIssuer;
    type MinReceivedCertToBeAbleToIssueCert = SmithsMinReceivedCertToBeAbleToIssueCert;
    type OnNewcert = SmithsSubWot;
    type OnRemovedCert = SmithsSubWot;
    type ValidityPeriod = SmithsValidityPeriod;
}

pub const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(
    initial_identities_len: usize,
    initial_smiths_len: usize,
) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
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
                    },
                )
            })
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_certification::GenesisConfig::<Test, Instance1> {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: clique_wot(initial_identities_len, ValidityPeriod::get()),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_membership::GenesisConfig::<Test, Instance2> {
        memberships: (1..=initial_smiths_len)
            .map(|i| {
                (
                    i as u32,
                    sp_membership::MembershipData {
                        expire_on: SmithsMembershipPeriod::get(),
                    },
                )
            })
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_certification::GenesisConfig::<Test, Instance2> {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: clique_wot(initial_smiths_len, SmithsValidityPeriod::get()),
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
        SmithsSubWot::on_finalize(System::block_number());
        SmithsMembership::on_finalize(System::block_number());
        SmithsCert::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        DuniterWot::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
        Membership::on_initialize(System::block_number());
        Cert::on_initialize(System::block_number());
        SmithsSubWot::on_initialize(System::block_number());
        SmithsMembership::on_initialize(System::block_number());
        SmithsCert::on_initialize(System::block_number());
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
