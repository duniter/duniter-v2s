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
use crate::{
    Error, GenesisIdty, IdtyName, IdtyValue, NewOwnerKeyPayload, RevocationPayload,
    NEW_OWNER_KEY_PAYLOAD_PREFIX, REVOCATION_PAYLOAD_PREFIX,
};
use codec::Encode;
//use frame_support::assert_noop;
use frame_support::{assert_err, assert_ok};
use frame_system::{EventRecord, Phase};
use sp_runtime::testing::TestSignature;

type IdtyVal = IdtyValue<u64, u64, ()>;

fn alice() -> GenesisIdty<Test> {
    GenesisIdty {
        index: 1,
        name: IdtyName::from("Alice"),
        value: IdtyVal {
            data: (),
            next_creatable_identity_on: 0,
            old_owner_key: None,
            owner_key: 1,
            removable_on: 0,
            status: crate::IdtyStatus::Validated,
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
            owner_key: 2,
            removable_on: 0,
            status: crate::IdtyStatus::Validated,
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

#[test]
fn test_create_identity_ok() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Alice should be able to create an identity
        assert_ok!(Identity::create_identity(Origin::signed(1), 2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated {
                    idty_index: 2,
                    owner_key: 2,
                }),
                topics: vec![],
            }
        );
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
        assert_ok!(Identity::create_identity(Origin::signed(1), 2));

        // The identity shoud expire in blocs #3
        run_to_block(3);
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyRemoved { idty_index: 2 }),
                topics: vec![],
            }
        );

        // We shoud be able to recreate the identity
        run_to_block(4);
        assert_ok!(Identity::create_identity(Origin::signed(1), 2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated {
                    idty_index: 3,
                    owner_key: 2,
                }),
                topics: vec![],
            }
        );
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
        assert_ok!(Identity::create_identity(Origin::signed(1), 2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated {
                    idty_index: 2,
                    owner_key: 2,
                }),
                topics: vec![],
            }
        );
        assert_eq!(Identity::identity(1).unwrap().next_creatable_identity_on, 4);

        // Alice cannot create a new identity before block #4
        run_to_block(2);
        assert_eq!(
            Identity::create_identity(Origin::signed(1), 3),
            Err(Error::<Test>::NotRespectIdtyCreationPeriod.into())
        );

        // Alice should be able to create a second identity after block #4
        run_to_block(4);
        assert_ok!(Identity::create_identity(Origin::signed(1), 3));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated {
                    idty_index: 3,
                    owner_key: 3,
                }),
                topics: vec![],
            }
        );
    });
}

#[test]
fn test_change_owner_key() {
    new_test_ext(IdentityConfig {
        identities: vec![alice(), bob()],
    })
    .execute_with(|| {
        let genesis_hash = System::block_hash(0);
        let old_owner_key = 1u64;
        let mut new_key_payload = NewOwnerKeyPayload {
            genesis_hash: &genesis_hash,
            idty_index: 1u64,
            old_owner_key: &old_owner_key,
        };

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Verify genesis data
        assert_eq!(System::sufficients(&1), 1);
        assert_eq!(System::sufficients(&10), 0);

        // Caller should have an associated identity
        assert_err!(
            Identity::change_owner_key(
                Origin::signed(42),
                10,
                TestSignature(10, (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode())
            ),
            Error::<Test>::IdtyIndexNotFound
        );

        // Payload must be signed by the new key
        assert_err!(
            Identity::change_owner_key(
                Origin::signed(1),
                10,
                TestSignature(42, (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode())
            ),
            Error::<Test>::InvalidNewOwnerKeySig
        );

        // Payload must be prefixed
        assert_err!(
            Identity::change_owner_key(
                Origin::signed(1),
                10,
                TestSignature(10, new_key_payload.encode())
            ),
            Error::<Test>::InvalidNewOwnerKeySig
        );

        // New owner key should not be used by another identity
        assert_err!(
            Identity::change_owner_key(
                Origin::signed(1),
                2,
                TestSignature(2, (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode())
            ),
            Error::<Test>::OwnerKeyAlreadyUsed
        );

        // Alice can change her owner key
        assert_ok!(Identity::change_owner_key(
            Origin::signed(1),
            10,
            TestSignature(10, (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode())
        ));
        assert_eq!(
            Identity::identity(1),
            Some(IdtyVal {
                data: (),
                next_creatable_identity_on: 0,
                old_owner_key: Some((1, 1)),
                owner_key: 10,
                removable_on: 0,
                status: crate::IdtyStatus::Validated,
            })
        );
        // Alice still sufficient
        assert_eq!(System::sufficients(&1), 1);
        // New owner key should become a sufficient account
        assert_eq!(System::sufficients(&10), 1);

        run_to_block(2);

        // Alice can't re-change her owner key too early
        new_key_payload.old_owner_key = &10;
        assert_err!(
            Identity::change_owner_key(
                Origin::signed(10),
                100,
                TestSignature(
                    100,
                    (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode()
                )
            ),
            Error::<Test>::OwnerKeyAlreadyRecentlyChanged
        );

        // Alice can re-change her owner key after ChangeOwnerKeyPeriod blocs
        run_to_block(2 + <Test as crate::Config>::ChangeOwnerKeyPeriod::get());
        assert_ok!(Identity::change_owner_key(
            Origin::signed(10),
            100,
            TestSignature(
                100,
                (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode()
            )
        ));
        // Old old owner key should not be sufficient anymore
        assert_eq!(System::sufficients(&1), 0);
        // Old owner key should still sufficient
        assert_eq!(System::sufficients(&10), 1);
        // New owner key should become a sufficient account
        assert_eq!(System::sufficients(&100), 1);

        // Revoke identity 1
        assert_ok!(Identity::revoke_identity(
            Origin::signed(42),
            1,
            100,
            TestSignature(
                100,
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
        // Old owner key should not be sufficient anymore
        assert_eq!(System::sufficients(&10), 0);
        // Last owner key should not be sufficient anymore
        assert_eq!(System::sufficients(&100), 0);
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
                Origin::signed(1),
                1,
                42,
                TestSignature(42, (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode())
            ),
            Err(Error::<Test>::InvalidRevocationKey.into())
        );

        // Payload must be prefixed
        assert_eq!(
            Identity::revoke_identity(
                Origin::signed(1),
                1,
                1,
                TestSignature(1, revocation_payload.encode())
            ),
            Err(Error::<Test>::InvalidRevocationSig.into())
        );

        // Anyone can submit a revocation payload
        assert_ok!(Identity::revoke_identity(
            Origin::signed(42),
            1,
            1,
            TestSignature(1, (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode())
        ));

        let events = System::events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::System(frame_system::Event::KilledAccount { account: 1 }),
                topics: vec![],
            }
        );
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyRemoved { idty_index: 1 }),
                topics: vec![],
            }
        );

        run_to_block(2);

        // The identity no longer exists
        assert_eq!(
            Identity::revoke_identity(
                Origin::signed(1),
                1,
                1,
                TestSignature(1, (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode())
            ),
            Err(Error::<Test>::IdtyNotFound.into())
        );
    });
}
