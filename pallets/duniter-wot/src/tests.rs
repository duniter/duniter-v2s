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

use crate::{mock::*, pallet as pallet_duniter_wot};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use pallet_identity::{
    IdtyName, IdtyStatus, RevocationPayload, RevocationReason, REVOCATION_PAYLOAD_PREFIX,
};
use pallet_membership::MembershipRemovalReason;
use sp_runtime::testing::TestSignature;

/// test that genesis builder creates the good number of identities
/// and good identity and certification metadate
#[test]
fn test_genesis_build() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);
        // Verify state
        assert_eq!(Identity::identities_count(), 3);
        assert_eq!(Identity::identity(1).unwrap().next_creatable_identity_on, 0);
        assert_eq!(
            pallet_certification::Pallet::<Test>::idty_cert_meta(1).next_issuable_on,
            2
        );
    });
}

/// test that Alice is not able to create an identity when she received too few certs (2 of 4)
#[test]
fn test_creator_not_allowed_to_create_idty() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Alice did not receive enough certs
        // (anyway Alice should not be able to create an identity before block #2
        // because Alice.next_issuable_on = 2)
        assert_noop!(
            Identity::create_identity(RuntimeOrigin::signed(1), 4),
            pallet_duniter_wot::Error::<Test>::NotEnoughReceivedCertsToCreateIdty
        );
    });
}

/// test that Alice is able to create an identity when she received enough certs (4)
#[test]
fn test_creator_allowed_to_create_idty() {
    new_test_ext(5).execute_with(|| {
        run_to_block(2);

        // Alice should be able to create an identity
        assert_ok!(
            Identity::create_identity(RuntimeOrigin::signed(1), 6),
            // pallet_duniter_wot::Error::<Test>::NotEnoughReceivedCertsToCreateIdty
        );
    });
}

/// test identity creation and that a first cert is emitted
#[test]
fn test_create_idty_ok() {
    new_test_ext(5).execute_with(|| {
        run_to_block(2);

        // Alice should be able to create an identity at block #2
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));
        // 2 events should have occurred: IdtyCreated and CertAdded
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyCreated {
                idty_index: 6,
                owner_key: 6,
            },
        ));
        System::assert_has_event(RuntimeEvent::Cert(pallet_certification::Event::CertAdded {
            issuer: 1,
            receiver: 6,
        }));

        assert_eq!(
            Identity::identity(6).unwrap().status,
            IdtyStatus::Unconfirmed
        );
        assert_eq!(Identity::identity(6).unwrap().next_scheduled, 2 + 2);
    });
}

/// test identity validation
#[test]
fn test_new_idty_validation() {
    new_test_ext(5).execute_with(|| {
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
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(2), 6));
        System::assert_has_event(RuntimeEvent::Cert(pallet_certification::Event::CertAdded {
            issuer: 2,
            receiver: 6,
        }));

        // Ferdie should be able to claim membership
        run_to_block(5);
        assert_ok!(Membership::try_add_membership(6));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipAdded {
                member: 6,
                expire_on: 5 + <Test as pallet_membership::Config>::MembershipPeriod::get(),
            },
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
                next_scheduled: 0,
                status: IdtyStatus::Member,
            })
        );
    });
}

/// test that Ferdie can confirm an identity created for him by Alice
#[test]
fn test_confirm_idty_ok() {
    new_test_ext(5).execute_with(|| {
        run_to_block(2);

        // Alice creates Ferdie identity
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));

        run_to_block(3);

        // Ferdie should be able to confirm his identity
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(6),
            IdtyName::from("Ferdie"),
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyConfirmed {
                idty_index: 6,
                name: IdtyName::from("Ferdie"),
            },
        ));
    });
}

/// test identity revocation
/// - anyone can submit a revocation certificate signed by bob
#[test]
fn test_revoke_idty() {
    new_test_ext(5).execute_with(|| {
        run_to_block(2);

        // Alice identity can be revoked
        assert_ok!(Identity::revoke_identity(
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
        ));
        // her membership should be removed
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 1,
                reason: pallet_membership::MembershipRemovalReason::Revoked,
            },
        ));

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
            pallet_identity::Event::IdtyRevoked {
                idty_index: 2,
                reason: pallet_identity::RevocationReason::User,
            },
        ));
    });
}

/// test that expired membership lose the identity after a delay
#[test]
fn test_idty_membership_expire() {
    new_test_ext(3).execute_with(|| {
        run_to_block(4);

        // Alice renews her membership
        assert_ok!(Membership::try_renew_membership(1));
        // Bob renews his membership
        assert_ok!(Membership::try_renew_membership(2));

        run_to_block(5);
        // renew certifications so that Alice can still issue cert at block 22
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(2), 1));
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(3), 1));

        // Charlie's membership should expire at block #8
        run_to_block(8);
        assert_ok!(Membership::try_renew_membership(1));
        assert!(Membership::membership(3).is_none());

        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 3,
                reason: MembershipRemovalReason::Expired,
            },
        ));
        assert_eq!(
            Identity::identity(3),
            Some(pallet_identity::IdtyValue {
                data: (),
                next_creatable_identity_on: 0,
                old_owner_key: None,
                owner_key: 3,
                next_scheduled: 14, // = 8 (membership removal block) + 6 (auto revocation period)
                status: IdtyStatus::NotMember,
            })
        );
        // check that identity is added to auto-revoke list (currently IdentityChangeSchedule)
        assert_eq!(Identity::next_scheduled(14), vec!(3));
        run_to_block(14);
        assert_ok!(Membership::try_renew_membership(1));
        // Charlie's identity should be auto-revoked at block #11 (8 + 3)
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRevoked {
                idty_index: 3,
                reason: pallet_identity::RevocationReason::Expired,
            },
        ));
        assert_eq!(
            Identity::identity(3),
            Some(pallet_identity::IdtyValue {
                data: (),
                next_creatable_identity_on: 0,
                old_owner_key: None,
                owner_key: 3,
                next_scheduled: 21, // = 14 (revocation block) + 7 (deletion period)
                status: IdtyStatus::Revoked,
            })
        );
        // Alice can't certify revoked identity
        assert_noop!(
            Cert::add_cert(RuntimeOrigin::signed(1), 3),
            pallet_duniter_wot::Error::<Test>::TargetStatusInvalid
        );

        run_to_block(21);
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved {
                idty_index: 3,
                reason: pallet_identity::RemovalReason::Revoked,
            },
        ));
        // Alice can't certify removed identity
        assert_noop!(
            Cert::add_cert(RuntimeOrigin::signed(1), 3),
            pallet_duniter_wot::Error::<Test>::IdtyNotFound
        );
    });
}

/// when an identity is confirmed and not validated, the certification received should be removed
#[test]
fn test_unvalidated_idty_certs_removal() {
    new_test_ext(5).execute_with(|| {
        // Alice creates Ferdie identity
        run_to_block(2);
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 6));

        // Ferdie confirms his identity
        run_to_block(3);
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(6),
            IdtyName::from("Ferdie"),
        ));

        assert_eq!(Cert::certs_by_receiver(6).len(), 1);
        // After ValidationPeriod, Ferdie identity should be automatically removed
        // and his received certifications should be removed
        run_to_block(8);
        assert_eq!(Cert::certs_by_receiver(6).len(), 0);
    });
}

/// test what happens when certification expire
#[test]
fn test_certification_expire() {
    new_test_ext(3).execute_with(|| {
        // smith cert Bob → Alice not renewed
        // cert Bob → Alice not renewed
        // --- BLOCK 2 ---
        run_to_block(2);
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(1), 2));
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(2), 3));
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(3), 1));
        // --- BLOCK 4 ---
        run_to_block(4);
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(1), 3));
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(3), 2));
        // --- BLOCK 7 ---
        run_to_block(7);
        assert_ok!(Membership::try_renew_membership(1));
        assert_ok!(Membership::try_renew_membership(2));
        assert_ok!(Membership::try_renew_membership(3));

        // --- BLOCK 14 ---
        run_to_block(14);
        assert_ok!(Membership::try_renew_membership(1));
        assert_ok!(Membership::try_renew_membership(2));
        assert_ok!(Membership::try_renew_membership(3));

        // normal cert Bob → Alice expires at block 20
        run_to_block(20);
        // println!("{:?}", System::events());
        System::assert_has_event(RuntimeEvent::Cert(
            pallet_certification::Event::CertRemoved {
                issuer: 2,   // Bob
                receiver: 1, // Alice
                expiration: true,
            },
        ));
        // in consequence, since Alice has only 1/2 normal certification remaining, she looses normal membership
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 1,
                reason: MembershipRemovalReason::NotEnoughCerts,
            },
        ));

        // --- BLOCK 21 ---
        // Bob and Charlie can renew their membership
        run_to_block(21);
        assert_ok!(Membership::try_renew_membership(2));
        assert_ok!(Membership::try_renew_membership(3));

        // Alice can not renew her membership which does not exist
        assert_noop!(
            Membership::try_renew_membership(1),
            pallet_membership::Error::<Test>::MembershipNotFound
        );

        // Alice can not claim her membership because she does not have enough certifications
        assert_noop!(
            Membership::try_add_membership(1),
            pallet_duniter_wot::Error::<Test>::NotEnoughCerts
        );

        // --- BLOCK 23 ---
        run_to_block(26);
        // println!("{:?}", System::events());
        // after a delay, the non member identity is automatically revoked
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRevoked {
                idty_index: 1,
                reason: RevocationReason::Expired,
            },
        ));
    })
}

/// test some cases where identity should not be able to issue cert
// - when source or target is not member (sub wot)
// - when source or target membership is pending (both wot)
#[test]
fn test_cert_can_not_be_issued() {
    new_test_ext(4).execute_with(|| {
        // smith cert Bob → Alice not renewed
        // cert Bob → Alice not renewed
        // --- BLOCK 2 ---
        run_to_block(2);
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(1), 2)); // +20
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(2), 3)); // +20
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(3), 4)); // +20
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(4), 1)); // +20
                                                                   // --- BLOCK 4 ---
        run_to_block(4);
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(2), 4)); // +20
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(3), 2)); // +20
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(4), 3)); // +20
                                                                   // --- BLOCK 7 ---
        run_to_block(7);
        assert_ok!(Membership::try_renew_membership(1)); // + 8
        assert_ok!(Membership::try_renew_membership(2)); // + 8
        assert_ok!(Membership::try_renew_membership(3)); // + 8
        assert_ok!(Membership::try_renew_membership(4)); // + 8

        run_to_block(14);
        assert_ok!(Membership::try_renew_membership(1)); // + 8
        assert_ok!(Membership::try_renew_membership(2)); // + 8
        assert_ok!(Membership::try_renew_membership(3)); // + 8
        assert_ok!(Membership::try_renew_membership(4)); // + 8

        run_to_block(20);
        // println!("{:?}", System::events());
        System::assert_has_event(RuntimeEvent::Cert(
            pallet_certification::Event::CertRemoved {
                issuer: 2,   // Bob
                receiver: 1, // Alice
                expiration: true,
            },
        ));
        // other certifications expire, but not Dave → Alice
        // in consequence, since Alice has only 1/2 certification remaining, she looses membership
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 1,
                reason: MembershipRemovalReason::NotEnoughCerts,
            }, // pending membership expires at 23
        ));

        run_to_block(21);
        // println!("{:?}", System::events());
        // Charlie certifies Alice so she again has enough certs
        assert_ok!(Cert::add_cert(RuntimeOrigin::signed(3), 1));
        assert_ok!(Cert::renew_cert(RuntimeOrigin::signed(4), 1));
        // renew
        // Alice did not claim membership, she is not member
        // but her cert delay has been reset (→ 23)
        assert_eq!(Membership::membership(1), None);

        // run_to_block(23);
        // if identity of alice was not removed because pending for too long
        // she would have been able to emit a cert without being member
    })
}
