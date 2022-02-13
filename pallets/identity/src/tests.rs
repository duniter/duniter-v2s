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
use crate::{Error, GenesisIdty, IdtyName, IdtyValue};
//use frame_support::assert_err;
use frame_support::assert_ok;
use frame_system::{EventRecord, Phase};

type IdtyVal = IdtyValue<u64, u64>;

fn alice() -> GenesisIdty<Test> {
    GenesisIdty {
        index: 1,
        name: IdtyName::from("Alice"),
        value: IdtyVal {
            next_creatable_identity_on: 0,
            owner_key: 1,
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

        // Alice should be able te create an identity
        assert_ok!(Identity::create_identity(Origin::signed(1), 2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated(2, 2)),
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
        assert_ok!(Identity::create_identity(Origin::signed(1), 2));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated(2, 2)),
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

        // Alice should be able te create a second identity after block #4
        run_to_block(4);
        assert_ok!(Identity::create_identity(Origin::signed(1), 3));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyCreated(3, 3)),
                topics: vec![],
            }
        );
    });
}
