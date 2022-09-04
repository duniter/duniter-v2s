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

use crate::mock::*;
use crate::mock::{Identity, System};
use codec::Encode;
use frame_support::instances::Instance1;
use frame_support::{assert_noop, assert_ok};
use frame_system::{EventRecord, Phase};
use pallet_identity::{
    IdtyName, IdtyStatus, NewOwnerKeyPayload, RevocationPayload, NEW_OWNER_KEY_PAYLOAD_PREFIX,
    REVOCATION_PAYLOAD_PREFIX,
};
use sp_runtime::testing::TestSignature;

#[test]
fn test_genesis_build() {
    new_test_ext(3, 2).execute_with(|| {
        run_to_block(1);
        // Verify state
        assert_eq!(Identity::identities_count(), 3);
        assert_eq!(Identity::identity(1).unwrap().next_creatable_identity_on, 0);
        assert_eq!(
            pallet_certification::Pallet::<Test, Instance1>::idty_cert_meta(1).next_issuable_on,
            2
        );
    });
}

#[test]
fn test_creator_not_allowed_to_create_idty() {
    new_test_ext(3, 2).execute_with(|| {
        run_to_block(1);

        // Alice should not be able to create an identity before block #2
        // because Alice.next_issuable_on = 2
        assert_noop!(
            Identity::create_identity(Origin::signed(1), 4),
            pallet_identity::Error::<Test>::CreatorNotAllowedToCreateIdty
        );
    });
}

#[test]
fn test_join_smiths() {
    new_test_ext(5, 3).execute_with(|| {
        run_to_block(2);

        // Dave shoud be able to request smith membership
        assert_ok!(SmithsMembership::request_membership(Origin::signed(4), ()));
        System::assert_has_event(Event::SmithsMembership(
            pallet_membership::Event::MembershipRequested(4),
        ));

        // Then, Alice should be able to send a smith cert to Dave
        run_to_block(3);
        assert_ok!(SmithsCert::add_cert(Origin::signed(1), 1, 4));

        // Then, Bob should be able to send a smith cert to Dave
        run_to_block(4);
        assert_ok!(SmithsCert::add_cert(Origin::signed(2), 2, 4));

        // Then, Dave should be able to claim his membership
        run_to_block(4);
        assert_ok!(SmithsMembership::claim_membership(
            Origin::signed(4),
            Some(4)
        ));
        System::assert_has_event(Event::SmithsMembership(
            pallet_membership::Event::MembershipAcquired(4),
        ));
    });
}

#[test]
fn test_smith_certs_expirations_should_revoke_smith_membership() {
    new_test_ext(5, 3).execute_with(|| {
        // After block #10, alice membership should be revoked due to smith certs expiration
        run_to_block(10);
        System::assert_has_event(Event::SmithsMembership(
            pallet_membership::Event::MembershipRevoked(1),
        ));
    });
}

#[test]
fn test_smith_member_cant_change_its_idty_address() {
    new_test_ext(5, 3).execute_with(|| {
        run_to_block(2);

        let genesis_hash = System::block_hash(0);
        let new_key_payload = NewOwnerKeyPayload {
            genesis_hash: &genesis_hash,
            idty_index: 3u32,
            old_owner_key: &3u64,
        };

        // Identity 3 can't change it's address
        assert_noop!(
            Identity::change_owner_key(
                Origin::signed(3),
                13,
                TestSignature(13, (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode())
            ),
            pallet_identity::Error::<Test>::NotAllowedToChangeIdtyAddress
        );
    });
}

#[test]
fn test_smith_member_cant_revoke_its_idty() {
    new_test_ext(5, 3).execute_with(|| {
        run_to_block(2);

        let revocation_payload = RevocationPayload {
            idty_index: 3u32,
            genesis_hash: System::block_hash(0),
        };

        // Identity 3 can't change it's address
        assert_noop!(
            Identity::revoke_identity(
                Origin::signed(3),
                3,
                3,
                TestSignature(3, (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode())
            ),
            pallet_identity::Error::<Test>::NotAllowedToRemoveIdty
        );
    });
}

#[test]
fn test_revoke_smiths_them_rejoin() {
    new_test_ext(5, 4).execute_with(|| {
        run_to_block(2);

        // Dave shoud be able to revoke his smith membership
        assert_ok!(SmithsMembership::revoke_membership(
            Origin::signed(4),
            Some(4)
        ));

        // Dave should not be able to re-request membership before the RevocationPeriod end
        run_to_block(3);
        assert_noop!(
            SmithsMembership::request_membership(Origin::signed(4), ()),
            pallet_membership::Error::<Test, crate::Instance2>::MembershipRevokedRecently
        );

        // At block #6, Dave shoud be able to request smith membership
        run_to_block(6);
        assert_ok!(SmithsMembership::request_membership(Origin::signed(4), ()));

        // Then, Alice should be able to send a smith cert to Dave
        assert_ok!(SmithsCert::add_cert(Origin::signed(1), 1, 4));
    });
}

#[test]
fn test_create_idty_ok() {
    new_test_ext(5, 2).execute_with(|| {
        run_to_block(2);

        // Alice should be able to create an identity at block #2
        assert_ok!(Identity::create_identity(Origin::signed(1), 6));
        // 2 events should have occurred: IdtyCreated and NewCert
        let events = System::events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyCreated {
                    idty_index: 6,
                    owner_key: 6,
                }),
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
    });
}

#[test]
fn test_new_idty_validation() {
    new_test_ext(5, 2).execute_with(|| {
        // Alice creates Ferdie identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(Origin::signed(1), 6));

        // Ferdie confirms his identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            Origin::signed(6),
            IdtyName::from("Ferdie"),
        ));

        // Bob should be able to certify Ferdie
        run_to_block(4);
        assert_ok!(Cert::add_cert(Origin::signed(2), 2, 6));
        System::assert_has_event(Event::Cert(pallet_certification::Event::NewCert {
            issuer: 2,
            issuer_issued_count: 5,
            receiver: 6,
            receiver_received_count: 2,
        }));

        // Anyone should be able to validate Ferdie identity
        run_to_block(5);
        assert_ok!(Identity::validate_identity(Origin::signed(42), 6));
        System::assert_has_event(Event::Membership(
            pallet_membership::Event::MembershipAcquired(6),
        ));
        System::assert_has_event(Event::Identity(pallet_identity::Event::IdtyValidated {
            idty_index: 6,
        }));

        // After PendingMembershipPeriod, Ferdie identity should not expire
        run_to_block(6);
        assert_eq!(
            Identity::identity(6),
            Some(pallet_identity::IdtyValue {
                data: (),
                next_creatable_identity_on: 0,
                old_owner_key: None,
                owner_key: 6,
                removable_on: 0,
                status: IdtyStatus::Validated,
            })
        );
    });
}

#[test]
fn test_confirm_idty_ok() {
    new_test_ext(5, 2).execute_with(|| {
        run_to_block(2);

        // Alice creates Ferdie identity
        assert_ok!(Identity::create_identity(Origin::signed(1), 6));

        run_to_block(3);

        // Ferdie should be able to confirm his identity
        assert_ok!(Identity::confirm_identity(
            Origin::signed(6),
            IdtyName::from("Ferdie"),
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
                event: Event::Identity(pallet_identity::Event::IdtyConfirmed {
                    idty_index: 6,
                    owner_key: 6,
                    name: IdtyName::from("Ferdie"),
                }),
                topics: vec![],
            }
        );
    });
}

#[test]
fn test_idty_membership_expire_them_requested() {
    new_test_ext(3, 2).execute_with(|| {
        run_to_block(4);

        // Alice renews her membership
        assert_ok!(Membership::renew_membership(Origin::signed(1), None));
        // Bob renews his membership
        assert_ok!(Membership::renew_membership(Origin::signed(2), None));

        // Charlie's membership should expire at block #8
        run_to_block(8);
        assert!(Membership::membership(3).is_none());
        let events = System::events();
        println!("{:?}", events);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Membership(pallet_membership::Event::MembershipExpired(3)),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 3 }),
                topics: vec![],
            }
        );

        // Charlie's identity should be removed at block #8
        assert!(Identity::identity(3).is_none());

        // Alice can't renew her cert to Charlie
        assert_noop!(
            Cert::add_cert(Origin::signed(1), 1, 3),
            pallet_certification::Error::<Test, Instance1>::CertNotAllowed
        );
    });
}

#[test]
fn test_unvalidated_idty_certs_removal() {
    new_test_ext(5, 2).execute_with(|| {
        // Alice creates Ferdie identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(Origin::signed(1), 6));

        // Ferdie confirms his identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            Origin::signed(6),
            IdtyName::from("Ferdie"),
        ));

        // After PendingMembershipPeriod, Ferdie identity should expire
        // and his received certifications should be removed
        assert_eq!(Cert::certs_by_receiver(6).len(), 1);
        run_to_block(6);
        assert_eq!(Cert::certs_by_receiver(6).len(), 0);
    });
}
