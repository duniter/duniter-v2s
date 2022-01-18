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
use crate::IdtyRight;
use frame_support::assert_err;
use frame_support::assert_ok;
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
    new_test_ext(3).execute_with(|| {
        run_to_block(2);

        // Alice should be able te create an identity at block #2
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Dave"),
            4
        ));
        // 2 events should have occurred: IdtyCreated and NewCert
        let events = System::events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyCreated(
                    IdtyName::from("Dave"),
                    4
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
                    issuer_issued_count: 3,
                    receiver: 4,
                    receiver_received_count: 1
                }),
                topics: vec![],
            }
        );
        assert_eq!(Identity::identity(4).unwrap().status, IdtyStatus::Created);
        assert_eq!(Identity::identity(4).unwrap().removable_on, 4);
    });
}

#[test]
fn test_ud_right_achievement_ok() {
    new_test_ext(3).execute_with(|| {
        // Alice create Dave identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Dave"),
            4
        ));

        // Dave confirm it's identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            Origin::signed(4),
            IdtyName::from("Dave"),
            4
        ));

        // Bob should be able to certify Dave
        run_to_block(4);
        assert_ok!(Cert::add_cert(Origin::signed(2), 2, 4));

        let events = System::events();
        // 3 events should have occurred: NewCert, MembershipAcquired, IdtyValidated and IdtyAcquireRight
        assert_eq!(events.len(), 4);
        println!("{:?}", events[2]);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Cert(pallet_certification::Event::NewCert {
                    issuer: 2,
                    issuer_issued_count: 3,
                    receiver: 4,
                    receiver_received_count: 2
                }),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipAcquired(4)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[2],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyValidated(IdtyName::from(
                    "Dave"
                ),)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[3],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyAcquireRight(
                    IdtyName::from("Dave"),
                    IdtyRight::Ud
                )),
                topics: vec![],
            }
        );
    });
}

#[test]
fn test_confirm_idty_ok() {
    new_test_ext(3).execute_with(|| {
        run_to_block(2);

        // Alice create Dave identity
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Dave"),
            4
        ));

        run_to_block(3);

        // Dave should be able to confirm it's identity
        assert_ok!(Identity::confirm_identity(
            Origin::signed(4),
            IdtyName::from("Dave"),
            4
        ));
        let events = System::events();
        // 2 events should have occurred: MembershipRequested and IdtyConfirmed
        assert_eq!(events.len(), 2);
        //println!("{:?}", events[0]);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipRequested(4)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyConfirmed(IdtyName::from(
                    "Dave"
                ),)),
                topics: vec![],
            }
        );
    });
}
