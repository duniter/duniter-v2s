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

use crate::mock::IdtyRight as Right;
use crate::mock::*;
use crate::{Error, IdtyName, IdtyValue};
use frame_support::assert_err;
use frame_support::assert_ok;
use frame_system::{EventRecord, Phase};

type IdtyVal = IdtyValue<u64, u64, (), Right>;

fn alice() -> IdtyVal {
    IdtyVal {
        data: (),
        owner_key: 1,
        name: IdtyName::from("Alice"),
        next_creatable_identity_on: 0,
        removable_on: 0,
        rights: vec![(Right::Right1, None)],
        status: crate::IdtyStatus::Validated,
    }
}
fn bob() -> IdtyVal {
    IdtyVal {
        data: (),
        owner_key: 2,
        name: IdtyName::from("Bob"),
        next_creatable_identity_on: 0,
        removable_on: 0,
        rights: vec![(Right::Right2, Some(20))],
        status: crate::IdtyStatus::Validated,
    }
}

#[test]
fn test_no_identity() {
    new_test_ext(IdentityConfig {
        identities: Vec::with_capacity(0),
    })
    .execute_with(|| {
        assert_eq!(Identity::identities_count(), 0);
    });
}

#[test]
fn test_creator_not_exist() {
    new_test_ext(IdentityConfig {
        identities: Vec::with_capacity(0),
    })
    .execute_with(|| {
        assert_eq!(
            Identity::create_identity(Origin::signed(1), 1, IdtyName::from("bob"), 2),
            Err(Error::<Test>::CreatorNotExist.into())
        );
    });
}

#[test]
fn test_creator_not_have_right_to_create_identity() {
    new_test_ext(IdentityConfig {
        identities: vec![bob()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Bob not have right to create identities
        assert_eq!(
            Identity::create_identity(Origin::signed(2), 1, IdtyName::from("Charlie"), 3),
            Err(Error::<Test>::CreatorNotHaveRightToCreateIdty.into())
        );
    })
}

#[test]
fn test_creator_not_owner() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Someone try to create an identity pretending to be Alice
        assert_eq!(
            Identity::create_identity(Origin::signed(2), 1, IdtyName::from("Charlie"), 3),
            Err(Error::<Test>::RequireToBeOwner.into())
        );
    })
}

#[test]
fn test_create_identity_ok() {
    new_test_ext(IdentityConfig {
        identities: vec![alice()],
    })
    .execute_with(|| {
        // We need to initialize at least one block before any call
        run_to_block(1);

        // Alice should be able te create an identity
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("bob"),
            2
        ));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated(IdtyName::from("bob"), 2)),
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

        // Alice should be able te create an identity
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("bob"),
            2
        ));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated(IdtyName::from("bob"), 2)),
                topics: vec![],
            }
        );
        assert_eq!(Identity::identity(1).unwrap().next_creatable_identity_on, 4);

        // Alice cannot create a new identity before block #4
        run_to_block(2);
        assert_eq!(
            Identity::create_identity(Origin::signed(1), 1, IdtyName::from("Charlie"), 3),
            Err(Error::<Test>::NotRespectIdtyCreationPeriod.into())
        );

        // Alice should be able te create a second identity after block #4
        run_to_block(4);
        assert_ok!(Identity::create_identity(
            Origin::signed(1),
            1,
            IdtyName::from("Charlie"),
            3
        ));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated(IdtyName::from("Charlie"), 3)),
                topics: vec![],
            }
        );
    });
}

#[test]
fn test_two_identities() {
    let identities = vec![alice(), bob()];

    new_test_ext(IdentityConfig { identities }).execute_with(|| {
        // Should have two identities
        assert_eq!(Identity::identities_count(), 2);

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Add right Right1 for Alice
        // Should fail because Alice already have this right
        assert_err!(
            Identity::add_right(Origin::root(), 1, Right::Right1),
            Error::<Test>::RightAlreadyAdded
        );
        // Add right Right2 for alice
        // Should succes and trigger the correct event
        assert_ok!(Identity::add_right(Origin::root(), 1, Right::Right2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyAcquireRight(
                    IdtyName::from("Alice"),
                    Right::Right2
                )),
                topics: vec![],
            }
        );

        run_to_block(3);

        // Delete right Right2 for Bob
        // Should succes and trigger the correct event
        assert_ok!(Identity::del_right(Origin::root(), 2, Right::Right2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyLostRight(
                    IdtyName::from("Bob"),
                    Right::Right2
                )),
                topics: vec![],
            }
        );

        // The Bob identity has no more rights, the inactivity period must start to run
        let idty2 = Identity::identity(2).expect("idty not found");
        assert!(idty2.rights.is_empty());
        assert_eq!(idty2.removable_on, 7);
    });
}
