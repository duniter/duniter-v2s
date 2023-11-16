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

use crate::mock::{Identity, System};
use crate::pallet as pallet_duniter_wot;
use crate::{mock::*, IdtyRemovalWotReason};
use codec::Encode;
use frame_support::instances::{Instance1, Instance2};
use frame_support::{assert_noop, assert_ok};
use pallet_identity::{
    IdtyIndexAccountIdPayload, IdtyName, IdtyStatus, RevocationPayload,
    NEW_OWNER_KEY_PAYLOAD_PREFIX, REVOCATION_PAYLOAD_PREFIX,
};
use sp_runtime::testing::TestSignature;

/// test that genesis builder creates the good number of identities
/// and good identity and certification metadate
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

/// test that Alice is not able to create an identity when she received too few certs (2 of 4)
#[test]
fn test_creator_not_allowed_to_create_idty() {
    new_test_ext(3, 2).execute_with(|| {
        run_to_block(1);

        // Alice did not receive enough certs
        // (anyway Alice should not be able to create an identity before block #2
        // because Alice.next_issuable_on = 2)
        assert_noop!(
            Identity::create_identity(RuntimeOrigin::signed(1), 4),
            pallet_duniter_wot::Error::<Test, Instance1>::NotEnoughReceivedCertsToCreateIdty
        );
    });
}

/// test that Alice is able to create an identity when she received enough certs (4)
#[test]
fn test_creator_allowed_to_create_idty() {
    new_test_ext(5, 2).execute_with(|| {
        run_to_block(2);

        // Alice should be able to create an identity
        assert_ok!(
            Identity::create_identity(RuntimeOrigin::signed(1), 6),
            // pallet_duniter_wot::Error::<Test, Instance1>::NotEnoughReceivedCertsToCreateIdty
        );
    });
}

/// test smith joining workflow
#[test]
fn test_join_smiths() {
    new_test_ext(5, 3).execute_with(|| {
        run_to_block(2);

        // Dave shoud be able to request smith membership
        assert_ok!(SmithMembership::request_membership(RuntimeOrigin::signed(
            4
        ),));
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
        let new_key_payload = IdtyIndexAccountIdPayload {
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

/// test that Ferdie can confirm an identity created for him by Alice
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

/// test identity revocation
/// - a smith ca not revoke his identity
/// - anyone can submit a revocation certificate signed by bob
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
            pallet_identity::Event::IdtyRemoved {
                idty_index: 2,
                reason: pallet_identity::IdtyRemovalReason::<IdtyRemovalWotReason>::Revoked,
            },
        ));
    });
}

/// test that expired membership lose the identity after a delay
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
            == RuntimeEvent::Identity(pallet_identity::Event::IdtyRemoved {
                idty_index: 3,
                reason: pallet_identity::IdtyRemovalReason::Other(
                    IdtyRemovalWotReason::MembershipExpired
                )
            })));
        // it should be moved to pending membership instead
        assert!(Membership::pending_membership(3).is_some());

        // then pending membership should expire and identity should finally be removed
        run_to_block(11);
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::PendingMembershipExpired(3),
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved {
                idty_index: 3,
                reason: pallet_identity::IdtyRemovalReason::Other(
                    IdtyRemovalWotReason::MembershipExpired,
                ),
            },
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

/// when an identity is confirmed and not validated, the certification received should be removed
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

/// test what happens when certification expire
#[test]
fn test_certification_expire() {
    new_test_ext(3, 3).execute_with(|| {
        // smith cert Bob → Alice not renewed
        // cert Bob → Alice not renewed
        // --- BLOCK 2 ---
        run_to_block(2);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(1), 1, 2));
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(2), 2, 3));
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(3), 3, 1));
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(1), 1, 2));
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(2), 2, 3));
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(3), 3, 1));
        // --- BLOCK 4 ---
        run_to_block(4);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(1), 1, 3));
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(3), 3, 2));
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(1), 1, 3));
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(3), 3, 2));
        // --- BLOCK 7 ---
        run_to_block(7);
        assert_ok!(SmithMembership::renew_membership(RuntimeOrigin::signed(1)));
        assert_ok!(SmithMembership::renew_membership(RuntimeOrigin::signed(2)));
        assert_ok!(SmithMembership::renew_membership(RuntimeOrigin::signed(3)));
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(1)));
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(2)));
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(3)));

        // smith cert Bob → Alice expires at block 10
        run_to_block(10);
        // println!("{:?}", System::events());
        System::assert_has_event(RuntimeEvent::SmithCert(
            pallet_certification::Event::RemovedCert {
                issuer: 2,                  // Bob
                issuer_issued_count: 1,     // Bob → Charlie only
                receiver: 1,                // Alice
                receiver_received_count: 1, // Charlie → Alice only
                expiration: true,
            },
        ));
        // in consequence, since Alice has only 1/2 smith certification remaining, she looses smith membership
        System::assert_has_event(RuntimeEvent::SmithMembership(
            pallet_membership::Event::MembershipExpired(1),
        ));

        // --- BLOCK 14 ---
        run_to_block(14);
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(1)));
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(2)));
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(3)));

        // normal cert Bob → Alice expires at block 20
        run_to_block(20);
        // println!("{:?}", System::events());
        System::assert_has_event(RuntimeEvent::Cert(
            pallet_certification::Event::RemovedCert {
                issuer: 2,                  // Bob
                issuer_issued_count: 1,     // Bob → Charlie
                receiver: 1,                // Alice
                receiver_received_count: 1, // Charlie → Alice
                expiration: true,
            },
        ));
        // in consequence, since Alice has only 1/2 normal certification remaining, she looses normal membership
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipExpired(1),
        ));

        // --- BLOCK 21 ---
        // Bob and Charlie can renew their membership
        run_to_block(21);
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(2)));
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(3)));

        // Alice can not renew her membership which does not exist because it is pending
        assert_noop!(
            Membership::renew_membership(RuntimeOrigin::signed(1)),
            pallet_membership::Error::<Test, Instance1>::MembershipNotFound
        );

        // Alice can not claim her membership because she does not have enough certifications
        assert_noop!(
            Membership::claim_membership(RuntimeOrigin::signed(1)),
            pallet_duniter_wot::Error::<Test, Instance1>::NotEnoughCertsToClaimMembership
        );

        // --- BLOCK 23 ---
        run_to_block(23);
        // println!("{:?}", System::events());
        // after a delay (3 blocks), the pending membership finally expires
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::PendingMembershipExpired(1),
        ));
        // and the identity is removed
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved {
                idty_index: 1,
                reason: pallet_identity::IdtyRemovalReason::Other(
                    IdtyRemovalWotReason::MembershipExpired,
                ),
            },
        ));
    })
}

/// test some cases where identity should not be able to issue cert
// - when source or target is not member (sub wot)
// - when source or target membership is pending (both wot)
#[test]
fn test_cert_can_not_be_issued() {
    new_test_ext(4, 3).execute_with(|| {
        // smith cert Bob → Alice not renewed
        // cert Bob → Alice not renewed
        // --- BLOCK 2 ---
        run_to_block(2);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(1), 1, 2)); // +10
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(2), 2, 3)); // +10
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(3), 3, 1)); // +10
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(1), 1, 2)); // +20
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(2), 2, 3)); // +20
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(3), 3, 4)); // +20
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(4), 4, 1)); // +20
                                                                    // --- BLOCK 4 ---
        run_to_block(4);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(1), 1, 3)); // +10
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(3), 3, 2)); // +10
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(2), 2, 4)); // +20
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(3), 3, 2)); // +20
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(4), 4, 3)); // +20
                                                                    // --- BLOCK 7 ---
        run_to_block(7);
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(3), 3, 1)); // +10
        assert_ok!(SmithMembership::renew_membership(RuntimeOrigin::signed(1))); // +20
        assert_ok!(SmithMembership::renew_membership(RuntimeOrigin::signed(2))); // +20
        assert_ok!(SmithMembership::renew_membership(RuntimeOrigin::signed(3))); // +20
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(1))); // + 8
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(2))); // + 8
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(3))); // + 8
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(4))); // + 8

        // smith cert Bob → Alice expires at block 10
        run_to_block(10);
        // println!("{:?}", System::events());
        System::assert_has_event(RuntimeEvent::SmithCert(
            pallet_certification::Event::RemovedCert {
                issuer: 2,                  // Bob
                issuer_issued_count: 1,     // Bob → Charlie only
                receiver: 1,                // Alice
                receiver_received_count: 1, // Charlie → Alice only
                expiration: true,
            },
        ));
        // in consequence, since Alice has only 1/2 smith certification remaining, she looses smith membership
        System::assert_has_event(RuntimeEvent::SmithMembership(
            pallet_membership::Event::MembershipExpired(1),
        ));

        run_to_block(11);
        // /!\ COUNTERINTUITIVE BEHAVIOR
        // Dave should not be able to receive a smith cert since he did not request smith membership
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(3), 3, 4));
        // Bob renews his smith certification towards Alice
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(2), 2, 1));
        // now Alice has enough certs
        assert_eq!(
            SmithCert::idty_cert_meta(1),
            pallet_certification::IdtyCertMeta {
                issued_count: 2, // → Bob, → Charlie
                // since Alice got back to min number of received certs to be able to issue cert
                // her next_issuable_on was updated to block_number (11) + cert_period (2)
                next_issuable_on: 13, // 11 + 2
                received_count: 2     // ← Bob, ← Charlie
            }
        );

        // because Alice did not claim membership, she is not member at this point
        assert_eq!(SmithMembership::membership(1), None);
        run_to_block(13);
        // /!\ COUNTERINTUITIVE BEHAVIOR
        // Alice is not smith member, she should not be able to issue cert
        assert_ok!(SmithCert::add_cert(RuntimeOrigin::signed(1), 1, 2));

        run_to_block(14);
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(1))); // + 8
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(2))); // + 8
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(3))); // + 8
        assert_ok!(Membership::renew_membership(RuntimeOrigin::signed(4))); // + 8

        run_to_block(20);
        // println!("{:?}", System::events());
        System::assert_has_event(RuntimeEvent::Cert(
            pallet_certification::Event::RemovedCert {
                issuer: 2,                  // Bob
                issuer_issued_count: 2,     // depends of the order of cert expiration
                receiver: 1,                // Alice
                receiver_received_count: 2, // depends of the order of cert expiration
                expiration: true,
            },
        ));
        // other certifications expire, but not Dave → Alice
        // in consequence, since Alice has only 1/2 certification remaining, she looses membership
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipExpired(1), // pending membership expires at 23
        ));

        run_to_block(21);
        // println!("{:?}", System::events());
        // Charlie certifies Alice so she again has enough certs
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(3), 3, 1));
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(4), 4, 1)); // renew
                                                                    // Alice did not claim membership, she is not member
                                                                    // but her cert delay has been reset (→ 23)
        assert_eq!(Membership::membership(1), None);

        // run_to_block(23);
        // if identity of alice was not removed because pending for too long
        // she would have been able to emit a cert without being member
    })
}
