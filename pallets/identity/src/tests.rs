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
use crate::{Error, GenesisIdty, IdtyName, IdtyValue, RevocationPayload};
use codec::Encode;
//use frame_support::assert_noop;
use frame_support::assert_ok;
use frame_system::{EventRecord, Phase};
use sp_runtime::testing::TestSignature;

type IdtyVal = IdtyValue<u64, u64, ()>;

fn alice() -> GenesisIdty<Test> {
    GenesisIdty {
        index: 1,
        name: IdtyName::from("Alice"),
        value: IdtyVal {
            next_creatable_identity_on: 0,
            owner_key: 1,
            removable_on: 0,
            status: crate::IdtyStatus::Validated,
            data: (),
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
fn test_idty_revocation() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        let revocation_payload = RevocationPayload {
            owner_key: 1,
            genesis_hash: System::block_hash(0),
        };

        // Payload must be signed by the right identity
        assert_eq!(
            Identity::revoke_identity(
                Origin::signed(1),
                revocation_payload.clone(),
                TestSignature(42, revocation_payload.encode())
            ),
            Err(Error::<Test>::InvalidRevocationProof.into())
        );

        // Anyone can submit a revocation payload
        assert_ok!(Identity::revoke_identity(
            Origin::signed(42),
            revocation_payload.clone(),
            TestSignature(1, revocation_payload.encode())
        ));

        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
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
                revocation_payload.clone(),
                TestSignature(1, revocation_payload.encode())
            ),
            Err(Error::<Test>::IdtyNotFound.into())
        );
    });
}
