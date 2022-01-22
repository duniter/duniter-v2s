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

use crate::mock::Identity;
use crate::mock::*;
use crate::WotDiff;
use frame_support::assert_err;
use frame_support::assert_ok;
use frame_support::error::BadOrigin;
use frame_support::instances::Instance1;
use frame_system::{EventRecord, Phase};
use pallet_identity::{IdtyName, IdtyStatus};

#[test]
fn test_genesis_build() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);
        // Verify state
        assert_eq!(Identity::identities_count(), 3);
        assert_eq!(Identity::identity(1).unwrap().next_creatable_identity_on, 0);
        assert_eq!(
            pallet_certification::Pallet::<Test, Instance1>::idty_cert_meta(1)
                .unwrap()
                .next_issuable_on,
            2
        );
    });
}

#[test]
fn test_creator_not_allowed_to_create_idty() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Alice should not be able te create an identity before block #2
        // because Alice.next_issuable_on = 2
        assert_err!(
            Identity::create_identity(Origin::signed(1), 1, IdtyName::from("Dave"), 4),
            pallet_identity::Error::<Test>::CreatorNotAllowedToCreateIdty
        );
    });
}

#[test]
fn test_create_idty_ok() {
    new_test_ext(5).execute_with(|| {
        run_to_block(2);

        // Alice should be able te create an identity at block #2
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Ferdie"),
            6
        ));
        // 2 events should have occurred: IdtyCreated and NewCert
        let events = System::events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyCreated(
                    IdtyName::from("Ferdie"),
                    6
                )),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Cert(pallet_certification::Event::NewCert {
                    issuer: 1,
                    issuer_issued_count: 5,
                    receiver: 6,
                    receiver_received_count: 1
                }),
                topics: vec![],
            }
        );
        assert_eq!(Identity::identity(6).unwrap().status, IdtyStatus::Created);
        assert_eq!(Identity::identity(6).unwrap().removable_on, 4);
        assert!(DuniterWot::wot_diffs().is_empty());
    });
}

#[test]
fn test_new_idty_validation() {
    new_test_ext(5).execute_with(|| {
        // Alice create Ferdie identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Ferdie"),
            6
        ));

        // Ferdie confirm it's identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            Origin::signed(6),
            IdtyName::from("Ferdie"),
            6
        ));

        // Ferdie is not yet validated, so there should be no wot diff
        assert!(DuniterWot::wot_diffs().is_empty());

        // Bob should be able to certify Ferdie
        run_to_block(4);
        assert_ok!(Cert::add_cert(Origin::signed(2), 2, 6));

        let events = System::events();
        // 3 events should have occurred: NewCert, MembershipAcquired and IdtyValidated
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Cert(pallet_certification::Event::NewCert {
                    issuer: 2,
                    issuer_issued_count: 5,
                    receiver: 6,
                    receiver_received_count: 2
                }),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipAcquired(6)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[2],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyValidated(IdtyName::from(
                    "Ferdie"
                ),)),
                topics: vec![],
            }
        );

        // Ferdie has just been validated, so the wot diff should contain her entry and all her
        // certifications
        assert_eq!(
            DuniterWot::wot_diffs(),
            vec![
                WotDiff::AddNode(6),
                WotDiff::AddLink(1, 6),
                WotDiff::AddLink(2, 6)
            ]
        );
    });
}

#[test]
fn test_confirm_idty_ok() {
    new_test_ext(5).execute_with(|| {
        run_to_block(2);

        // Alice create Ferdie identity
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Ferdie"),
            6
        ));

        run_to_block(3);

        // Ferdie should be able to confirm it's identity
        assert_ok!(Identity::confirm_identity(
            Origin::signed(6),
            IdtyName::from("Ferdie"),
            6
        ));
        let events = System::events();
        // 2 events should have occurred: MembershipRequested and IdtyConfirmed
        assert_eq!(events.len(), 2);
        //println!("{:?}", events[0]);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipRequested(6)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyConfirmed(IdtyName::from(
                    "Ferdie"
                ),)),
                topics: vec![],
            }
        );
    });
}

#[test]
fn test_idty_membership_expire_them_requested() {
    new_test_ext(3).execute_with(|| {
        run_to_block(4);

        // Alice renew her membership
        assert_ok!(Membership::renew_membership(Origin::signed(1), 1));
        // Bob renew his membership
        assert_ok!(Membership::renew_membership(Origin::signed(2), 2));

        // Charlie's membership should expire at block #5
        run_to_block(5);
        assert!(Membership::membership(3).is_none());
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipExpired(3)),
                topics: vec![],
            }
        );
        assert_eq!(DuniterWot::wot_diffs(), vec![WotDiff::DisableNode(3),]);

        // Charlie's identity should be disabled at block #5
        assert_eq!(Identity::identity(3).unwrap().status, IdtyStatus::Disabled);

        // Alice can't renew it's cert to Charlie
        assert_err!(Cert::add_cert(Origin::signed(1), 1, 3), BadOrigin);

        // Charlie should be able to request membership
        run_to_block(6);
        assert_ok!(Membership::request_membership(Origin::signed(3), 3, ()));

        // Charlie should re-enter in the wot immediatly
        let events = System::events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipRequested(3)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipAcquired(3)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[2],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyValidated(IdtyName::from(
                    "Charlie"
                ))),
                topics: vec![],
            }
        );

        assert_eq!(DuniterWot::wot_diffs(), vec![WotDiff::AddNode(3),]);
    });
}
