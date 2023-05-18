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

use crate::mock::*;
use crate::mock::{Identity, System};
use crate::pallet as pallet_duniter_wot;
use codec::Encode;
use frame_support::instances::{Instance1, Instance2};
use frame_support::{assert_noop, assert_ok};
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
        // but the true reason is that alice did not receive enough certs
        assert_noop!(
            Identity::create_identity(RuntimeOrigin::signed(1), 4),
            pallet_duniter_wot::Error::<Test, Instance1>::NotEnoughReceivedCertsToCreateIdty
        );
    });
}

/// test smith joining workflow
#[test]
fn test_join_smiths() {
    new_test_ext(5, 3).execute_with(|| {
        run_to_block(2);

        // Dave shoud be able to request smith membership
        assert_ok!(SmithMembership::request_membership(
            RuntimeOrigin::signed(4),
            ()
        ));
        System::assert_has_event(RuntimeEvent::SmithMembership(
            pallet_membership::Event::MembershipRequested(4),
        ));

        // Then, Alice should be able to send a smith cert to Dave
        run_to_block(3);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(1), 1, 4));

        // Then, Bob should be able to send a smith cert to Dave
        run_to_block(4);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(2), 2, 4));

        // Then, Dave should be able to claim his membership
        run_to_block(4);
        assert_ok!(SmithMembership::claim_membership(RuntimeOrigin::signed(4),));
        System::assert_has_event(RuntimeEvent::SmithMembership(
            pallet_membership::Event::MembershipAcquired(4),
        ));
    });
}

/// test smith membership expiry after cert expiration
#[test]
fn test_smith_certs_expirations_should_expire_smith_membership() {
    new_test_ext(5, 3).execute_with(|| {
        // After block #10, alice membership should be revoked due to smith certs expiration
        run_to_block(10);
        System::assert_has_event(RuntimeEvent::SmithMembership(
            pallet_membership::Event::MembershipExpired(1),
        ));
    });
}

/// test that smith can not change owner key
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
                RuntimeOrigin::signed(3),
                13,
                TestSignature(13, (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode())
            ),
            pallet_duniter_wot::Error::<Test, Instance2>::NotAllowedToChangeIdtyAddress
        );
    });
}

/// members of the smith subwot can not remove their identity
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
                RuntimeOrigin::signed(3),
                3,
                3,
                TestSignature(3, (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode())
            ),
            pallet_duniter_wot::Error::<Test, Instance2>::NotAllowedToRemoveIdty
        );
    });
}

/// test identity creation and that a first cert is emitted
#[test]
fn test_create_idty_ok() {
    new_test_ext(5, 2).execute_with(|| {
        run_to_block(2);

        // Alice should be able to create an identity at block #2
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));
        // 2 events should have occurred: IdtyCreated and NewCert
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyCreated {
                idty_index: 6,
                owner_key: 6,
            },
        ));
        System::assert_has_event(RuntimeEvent::Cert(pallet_certification::Event::NewCert {
            issuer: 1,
            issuer_issued_count: 5,
            receiver: 6,
            receiver_received_count: 1,
        }));

        assert_eq!(Identity::identity(6).unwrap().status, IdtyStatus::Created);
        assert_eq!(Identity::identity(6).unwrap().removable_on, 4);
    });
}

/// test identity validation
#[test]
fn test_new_idty_validation() {
    new_test_ext(5, 2).execute_with(|| {
        // Alice creates Ferdie identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));

        // Ferdie confirms his identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(6),
            IdtyName::from("Ferdie"),
        ));

        // Bob should be able to certify Ferdie
        run_to_block(4);
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(2), 2, 6));
        System::assert_has_event(RuntimeEvent::Cert(pallet_certification::Event::NewCert {
            issuer: 2,
            issuer_issued_count: 5,
            receiver: 6,
            receiver_received_count: 2,
        }));

        // Ferdie should not be able to claim membership
        // assert_noop!(
        //     Membership::claim_membership(RuntimeOrigin::signed(6)),
        //     pallet_membership::Error::<Test, Instance1>::xxx
        // );

        // Anyone should be able to validate Ferdie identity
        run_to_block(5);
        assert_ok!(Identity::validate_identity(RuntimeOrigin::signed(42), 6));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipAcquired(6),
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyValidated { idty_index: 6 },
        ));

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
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));

        run_to_block(3);

        // Ferdie should be able to confirm his identity
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(6),
            IdtyName::from("Ferdie"),
        ));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRequested(6),
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyConfirmed {
                idty_index: 6,
                owner_key: 6,
                name: IdtyName::from("Ferdie"),
            },
        ));
    });
}

#[test]
fn test_revoke_idty() {
    new_test_ext(5, 1).execute_with(|| {
        run_to_block(2);

        // Alice identity can not be revoked because she's smith
        assert_noop!(
            Identity::revoke_identity(
                RuntimeOrigin::signed(1),
                1,
                1,
                TestSignature(
                    1,
                    (
                        REVOCATION_PAYLOAD_PREFIX,
                        RevocationPayload {
                            idty_index: 1u32,
                            genesis_hash: System::block_hash(0),
                        }
                    )
                        .encode()
                )
            ),
            pallet_duniter_wot::Error::<Test, Instance2>::NotAllowedToRemoveIdty
        );

        // Anyone should be able to submit Bob revocation certificate
        assert_ok!(Identity::revoke_identity(
            RuntimeOrigin::signed(42),
            2,
            2,
            TestSignature(
                2,
                (
                    REVOCATION_PAYLOAD_PREFIX,
                    RevocationPayload {
                        idty_index: 2u32,
                        genesis_hash: System::block_hash(0),
                    }
                )
                    .encode()
            )
        ));

        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved { idty_index: 2 },
        ));
    });
}

/// test that expired membership lose the identity and can not be certified
#[test]
fn test_idty_membership_expire() {
    new_test_ext(3, 2).execute_with(|| {
        run_to_block(4);

        // Alice renews her membership
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(1)));
        // Bob renews his membership
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(2)));

        // Charlie's membership should expire at block #8
        run_to_block(8);
        assert!(Membership::membership(3).is_none());

        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipExpired(3),
        ));
        // membership expiry should not trigger identity removal
        assert!(!System::events().iter().any(|record| record.event
            == RuntimeEvent::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 3 })));
        // it should be moved to pending membership instead
        assert!(Membership::pending_membership(3).is_some());

        // then pending membership should expire and identity should finally be removed
        run_to_block(11);
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::PendingMembershipExpired(3),
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved { idty_index: 3 },
        ));

        // Charlie's identity should be removed at block #11
        assert!(Identity::identity(3).is_none());

        // Alice can't renew her cert to Charlie
        assert_noop!(
            Cert::add_cert(RuntimeOrigin::signed(1), 1, 3),
            pallet_duniter_wot::Error::<Test, Instance1>::IdtyNotFound
        );
    });
}

#[test]
fn test_unvalidated_idty_certs_removal() {
    new_test_ext(5, 2).execute_with(|| {
        // Alice creates Ferdie identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));

        // Ferdie confirms his identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(6),
            IdtyName::from("Ferdie"),
        ));

        // After PendingMembershipPeriod, Ferdie identity should expire
        // and his received certifications should be removed
        assert_eq!(Cert::certs_by_receiver(6).len(), 1);
        run_to_block(6);
        assert_eq!(Cert::certs_by_receiver(6).len(), 0);
    });
}
