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

use crate::{mock::*, *};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResultWithPostInfo};
use sp_core::{sr25519::Pair as KeyPair, Pair};
use sp_runtime::{MultiSignature, MultiSigner};

type IdtyVal = IdtyValue<u64, AccountId, ()>;

// Store the account id and the key pair to sign payload
struct Account {
    id: AccountId,
    signer: KeyPair,
}

// Create an Account given a u8
fn account(id: u8) -> Account {
    let pair = sp_core::sr25519::Pair::from_seed(&[id; 32]);
    Account {
        id: MultiSigner::Sr25519(pair.public()).into_account(),
        signer: pair,
    }
}

// Sign a payload using a key pair
fn test_signature(key_pair: KeyPair, payload: Vec<u8>) -> MultiSignature {
    MultiSignature::Sr25519(key_pair.sign(&payload))
}

fn alice() -> GenesisIdty<Test> {
    GenesisIdty {
        index: 1,
        name: IdtyName::from("Alice"),
        value: IdtyVal {
            data: (),
            next_creatable_identity_on: 0,
            old_owner_key: None,
            owner_key: account(1).id,
            next_scheduled: 0,
            status: crate::IdtyStatus::Member,
        },
    }
}

fn bob() -> GenesisIdty<Test> {
    GenesisIdty {
        index: 2,
        name: IdtyName::from("Bob"),
        value: IdtyVal {
            data: (),
            next_creatable_identity_on: 0,
            old_owner_key: None,
            owner_key: account(2).id,
            next_scheduled: 0,
            status: crate::IdtyStatus::Member,
        },
    }
}

fn inactive_bob() -> GenesisIdty<Test> {
    GenesisIdty {
        index: 2,
        name: IdtyName::from("Bob"),
        value: IdtyVal {
            data: (),
            next_creatable_identity_on: 0,
            old_owner_key: None,
            owner_key: account(2).id,
            next_scheduled: 2,
            status: crate::IdtyStatus::NotMember,
        },
    }
}

#[test]
fn test_no_identity() {
    new_test_ext(IdentityConfig {
        identities: Vec::new(),
    })
    .execute_with(|| {
        assert_eq!(Identity::identities_count(), 0);
    });
}

/// test that next identity index is correctly initialized and incremented
#[test]
fn test_identity_index() {
    new_test_ext(IdentityConfig {
        identities: vec![alice(), bob()],
    })
    .execute_with(|| {
        assert_eq!(Identity::identities_count(), 2);
        // assert_eq!(NextIdtyIndex, 3); // TODO check how to test that
        // ... create identity and check it was incremented
    });
}

#[test]
fn test_create_identity_ok() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Alice should be able to create an identity
        assert_ok!(Identity::create_identity(
            RuntimeOrigin::signed(account(1).id),
            account(2).id
        ));

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyCreated {
            idty_index: 2,
            owner_key: account(2).id,
        }));
    });
}

#[test]
fn test_create_identity_but_not_confirm_it() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Alice should be able to create an identity
        assert_ok!(Identity::create_identity(
            RuntimeOrigin::signed(account(1).id),
            account(2).id
        ));

        // The identity should expire in blocs #3
        run_to_block(3);

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyRemoved {
            idty_index: 2,
            reason: RemovalReason::Unconfirmed,
        }));

        // We shoud be able to recreate the identity
        run_to_block(4);
        assert_ok!(Identity::create_identity(
            RuntimeOrigin::signed(account(1).id),
            account(2).id
        ));

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyCreated {
            idty_index: 3,
            owner_key: account(2).id,
        }));
    });
}

#[test]
fn test_idty_creation_period() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Alice should be able to create an identity
        assert_ok!(Identity::create_identity(
            RuntimeOrigin::signed(account(1).id),
            account(2).id
        ));

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyCreated {
            idty_index: 2,
            owner_key: account(2).id,
        }));

        assert_eq!(Identity::identity(1).unwrap().next_creatable_identity_on, 4);

        // Alice cannot create a new identity before block #4
        run_to_block(2);
        assert_eq!(
            Identity::create_identity(RuntimeOrigin::signed(account(1).id), account(3).id),
            Err(Error::<Test>::NotRespectIdtyCreationPeriod.into())
        );

        // Alice should be able to create a second identity after block #4
        run_to_block(4);
        assert_ok!(Identity::create_identity(
            RuntimeOrigin::signed(account(1).id),
            account(3).id
        ));

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyCreated {
            idty_index: 3,
            owner_key: account(3).id,
        }));
    });
}

//
#[test]
fn test_change_owner_key() {
    new_test_ext(IdentityConfig {
        identities: vec![alice(), bob()],
    })
    .execute_with(|| {
        let genesis_hash = System::block_hash(0);
        let old_owner_key = account(1).id;
        let mut new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: 1u64,
            old_owner_key: &old_owner_key,
        };

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Verify genesis data
        assert_eq!(System::sufficients(&account(1).id), 1);
        assert_eq!(System::sufficients(&account(10).id), 0);
        // Caller should have an associated identity
        assert_noop!(
            Identity::change_owner_key(
                RuntimeOrigin::signed(account(42).id),
                account(10).id,
                test_signature(
                    account(10).signer,
                    (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload.clone()).encode()
                )
            ),
            Error::<Test>::IdtyIndexNotFound
        );
        // Payload must be signed by the new key
        assert_noop!(
            Identity::change_owner_key(
                RuntimeOrigin::signed(account(1).id),
                account(10).id,
                test_signature(
                    account(42).signer,
                    (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload.clone()).encode()
                )
            ),
            Error::<Test>::InvalidSignature
        );

        // Payload must be prefixed
        assert_noop!(
            Identity::change_owner_key(
                RuntimeOrigin::signed(account(1).id),
                account(10).id,
                test_signature(account(10).signer, new_key_payload.clone().encode())
            ),
            Error::<Test>::InvalidSignature
        );

        // New owner key should not be used by another identity
        assert_noop!(
            Identity::change_owner_key(
                RuntimeOrigin::signed(account(1).id),
                account(2).id,
                test_signature(
                    account(2).signer,
                    (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload.clone()).encode()
                )
            ),
            Error::<Test>::OwnerKeyAlreadyUsed
        );

        // Alice can change her owner key
        assert_ok!(Identity::change_owner_key(
            RuntimeOrigin::signed(account(1).id),
            account(10).id,
            test_signature(
                account(10).signer,
                (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload.clone()).encode()
            )
        ));
        assert_eq!(
            Identity::identity(1),
            Some(IdtyVal {
                data: (),
                next_creatable_identity_on: 0,
                old_owner_key: Some((account(1).id, 1)),
                owner_key: account(10).id,
                next_scheduled: 0,
                status: crate::IdtyStatus::Member,
            })
        );
        // Alice still sufficient
        assert_eq!(System::sufficients(&account(1).id), 1);
        // New owner key should become a sufficient account
        assert_eq!(System::sufficients(&account(10).id), 1);

        run_to_block(2);
        //
        // Alice can't re-change her owner key too early
        let old = account(10).id;
        new_key_payload.old_owner_key = &old;
        assert_noop!(
            Identity::change_owner_key(
                RuntimeOrigin::signed(account(10).id),
                account(100).id,
                test_signature(
                    account(100).signer,
                    (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload.clone()).encode()
                )
            ),
            Error::<Test>::OwnerKeyAlreadyRecentlyChanged
        );

        // Alice can re-change her owner key after ChangeOwnerKeyPeriod blocs
        run_to_block(2 + <Test as crate::Config>::ChangeOwnerKeyPeriod::get());
        assert_ok!(Identity::change_owner_key(
            RuntimeOrigin::signed(account(10).id),
            account(100).id,
            test_signature(
                account(100).signer,
                (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload.clone()).encode()
            )
        ));
        // Old old owner key should not be sufficient anymore
        assert_eq!(System::sufficients(&account(1).id), 0);
        // Old owner key should still sufficient
        assert_eq!(System::sufficients(&account(10).id), 1);
        // New owner key should become a sufficient account
        assert_eq!(System::sufficients(&account(100).id), 1);

        // Revoke identity 1
        assert_ok!(Identity::revoke_identity(
            RuntimeOrigin::signed(account(42).id),
            1,
            account(100).id,
            test_signature(
                account(100).signer,
                (
                    REVOCATION_PAYLOAD_PREFIX,
                    RevocationPayload {
                        idty_index: 1u64,
                        genesis_hash: System::block_hash(0),
                    }
                )
                    .encode()
            )
        ));
        // Old owner key is still sufficient (identity is revoked but not removed)
        assert_eq!(System::sufficients(&account(10).id), 1);
        // Last owner key should still be sufficient  (identity is revoked but not removed)
        assert_eq!(System::sufficients(&account(100).id), 1);
    });
}

// test link identity (does nothing because of AccountLinker type)
#[test]
fn test_link_account() {
    new_test_ext(IdentityConfig {
        identities: vec![alice(), bob()],
    })
    .execute_with(|| {
        let genesis_hash = System::block_hash(0);
        let account_id = account(10).id;
        let payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: 1u64,
            old_owner_key: &account_id,
        };

        run_to_block(1);

        // Caller should have an associated identity
        assert_noop!(
            Identity::link_account(
                RuntimeOrigin::signed(account(42).id),
                account(10).id,
                test_signature(
                    account(10).signer,
                    (LINK_IDTY_PAYLOAD_PREFIX, payload.clone()).encode()
                )
            ),
            Error::<Test>::IdtyIndexNotFound
        );
        // Payload must be signed by the new key
        assert_noop!(
            Identity::link_account(
                RuntimeOrigin::signed(account(1).id),
                account(10).id,
                test_signature(
                    account(42).signer,
                    (LINK_IDTY_PAYLOAD_PREFIX, payload.clone()).encode()
                )
            ),
            Error::<Test>::InvalidSignature
        );

        // Payload must be prefixed
        assert_noop!(
            Identity::link_account(
                RuntimeOrigin::signed(account(1).id),
                account(10).id,
                test_signature(account(10).signer, payload.clone().encode())
            ),
            Error::<Test>::InvalidSignature
        );

        // Alice can call link_account successfully
        assert_ok!(Identity::link_account(
            RuntimeOrigin::signed(account(1).id),
            account(10).id,
            test_signature(
                account(10).signer,
                (LINK_IDTY_PAYLOAD_PREFIX, payload.clone()).encode()
            )
        ));
    });
}

#[test]
fn test_idty_revocation_with_old_key() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        let genesis_hash = System::block_hash(0);
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: 1u64,
            old_owner_key: &account(1).id,
        };
        let revocation_payload = RevocationPayload {
            idty_index: 1u64,
            genesis_hash,
        };

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Change alice owner key
        assert_ok!(Identity::change_owner_key(
            RuntimeOrigin::signed(account(1).id),
            account(10).id,
            test_signature(
                account(10).signer,
                (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode()
            )
        ));
        assert!(Identity::identity(1).is_some());
        let idty_val = Identity::identity(1).unwrap();
        assert_eq!(idty_val.owner_key, account(10).id);
        assert_eq!(idty_val.old_owner_key, Some((account(1).id, 1)));

        // We should be able to revoke Alice identity with old key
        run_to_block(2);
        assert_ok!(Identity::revoke_identity(
            RuntimeOrigin::signed(account(42).id),
            1,
            account(1).id,
            test_signature(
                account(1).signer,
                (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode()
            )
        ));

        //run_to_block(2 + <Test as crate::Config>::ChangeOwnerKeyPeriod::get());
    });
}

#[test]
fn test_idty_revocation_with_old_key_after_old_key_expiration() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        let genesis_hash = System::block_hash(0);
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: 1u64,
            old_owner_key: &account(1).id,
        };
        let revocation_payload = RevocationPayload {
            idty_index: 1u64,
            genesis_hash,
        };

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Change alice owner key
        assert_ok!(Identity::change_owner_key(
            RuntimeOrigin::signed(account(1).id),
            account(10).id,
            test_signature(
                account(10).signer,
                (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode()
            )
        ));
        assert!(Identity::identity(1).is_some());
        let idty_val = Identity::identity(1).unwrap();
        assert_eq!(idty_val.owner_key, account(10).id);
        assert_eq!(idty_val.old_owner_key, Some((account(1).id, 1)));

        // We should not be able to revoke Alice identity with old key after ChangeOwnerKeyPeriod
        run_to_block(2 + <Test as crate::Config>::ChangeOwnerKeyPeriod::get());
        assert_noop!(
            Identity::revoke_identity(
                RuntimeOrigin::signed(account(42).id),
                1,
                account(1).id,
                test_signature(
                    account(1).signer,
                    (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode()
                )
            ),
            Error::<Test>::InvalidRevocationKey
        );
    });
}

#[test]
fn test_idty_revocation() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        let revocation_payload = RevocationPayload {
            idty_index: 1u64,
            genesis_hash: System::block_hash(0),
        };

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Payload must be signed by the right identity
        assert_eq!(
            Identity::revoke_identity(
                RuntimeOrigin::signed(account(1).id),
                1,
                account(42).id,
                test_signature(
                    account(42).signer,
                    (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode()
                )
            ),
            Err(Error::<Test>::InvalidRevocationKey.into())
        );

        // Payload must be prefixed
        assert_eq!(
            Identity::revoke_identity(
                RuntimeOrigin::signed(account(1).id),
                1,
                account(1).id,
                test_signature(account(1).signer, revocation_payload.encode())
            ),
            Err(Error::<Test>::InvalidSignature.into())
        );

        // Anyone can submit a revocation payload
        assert_ok!(Identity::revoke_identity(
            RuntimeOrigin::signed(account(42).id),
            1,
            account(1).id,
            test_signature(
                account(1).signer,
                (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode()
            )
        ));

        // // account is not killed anymore for revoked identity
        // System::assert_has_event(RuntimeEvent::System(frame_system::Event::KilledAccount {
        //     account: account(1).id,
        // }));
        // // identity is not removed immediately after revocation
        // System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyRemoved {
        //     idty_index: 1,
        //     reason: RemovalReason::Revoked,
        // }));
        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyRevoked {
            idty_index: 1,
            reason: RevocationReason::User,
        }));

        run_to_block(2);

        // The identity can not be revoked multiple times
        assert_eq!(
            Identity::revoke_identity(
                RuntimeOrigin::signed(account(1).id),
                1,
                account(1).id,
                test_signature(
                    account(1).signer,
                    (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode()
                )
            ),
            Err(Error::<Test>::AlreadyRevoked.into())
        );
    });
}
#[test]
fn test_inactive_genesis_members() {
    new_test_ext(IdentityConfig {
        identities: vec![alice(), inactive_bob()],
    })
    .execute_with(|| {
        let alice = alice();
        let bob = inactive_bob();
        assert!(pallet::Identities::<Test>::get(alice.index).is_some());
        assert!(pallet::Identities::<Test>::get(bob.index).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&alice.value.owner_key).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&bob.value.owner_key).is_some());

        run_to_block(2);

        // alice identity remains untouched
        assert!(pallet::Identities::<Test>::get(alice.index).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&alice.value.owner_key).is_some());
        // but bob identity has been revoked
        assert!(pallet::Identities::<Test>::get(bob.index).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&bob.value.owner_key).is_some());

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyRevoked {
            idty_index: bob.index,
            reason: RevocationReason::Expired,
        }));
    });
}

#[test]
fn test_revocation_of_genesis_member() {
    let alice = alice();
    let bob = inactive_bob();
    new_test_ext(IdentityConfig {
        identities: vec![alice.clone(), bob.clone()],
    })
    .execute_with(|| {
        assert!(pallet::Identities::<Test>::get(alice.index).is_some());
        assert!(pallet::Identities::<Test>::get(bob.index).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&alice.value.owner_key).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&bob.value.owner_key).is_some());

        // Necessary to go to block#1 to allow extrinsics consumption
        run_to_block(1);

        assert_ok!(revoke_self_identity(bob.clone()));

        // alice identity remains untouched
        assert!(pallet::Identities::<Test>::get(alice.index).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&alice.value.owner_key).is_some());
        // but bob identity has been revoked
        assert!(pallet::Identities::<Test>::get(bob.index).is_some());
        assert!(pallet::IdentityIndexOf::<Test>::get(&bob.value.owner_key).is_some());

        System::assert_has_event(RuntimeEvent::Identity(crate::Event::IdtyRevoked {
            idty_index: bob.index,
            reason: RevocationReason::User, // because called manually by revoke_self_identity
        }));
    });
}

fn revoke_self_identity(idty: GenesisIdty<Test>) -> DispatchResultWithPostInfo {
    Identity::revoke_identity(
        RuntimeOrigin::signed(account(idty.index as u8).id),
        idty.index,
        account(idty.index as u8).id,
        test_signature(
            account(idty.index as u8).signer,
            (
                REVOCATION_PAYLOAD_PREFIX,
                RevocationPayload {
                    idty_index: idty.index,
                    genesis_hash: System::block_hash(0),
                },
            )
                .encode(),
        ),
    )
}

// TODO add tests for all periods (confirmation, validation, autorevocation, deletion)
