// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use crate::mock::IdtyDid as Did;
use crate::mock::IdtyRight as Right;
use crate::mock::*;
use crate::Error;
use frame_support::assert_err;
use frame_support::assert_ok;
use frame_system::{EventRecord, Phase};
use std::collections::BTreeMap;

#[test]
fn test_no_identity() {
    let identities = BTreeMap::new();
    new_test_ext(IdentityConfig { identities }).execute_with(|| {
        assert_eq!(Identity::identities_count(), 0);
    });
}

#[test]
fn test_two_identities() {
    let mut identities = BTreeMap::new();
    identities.insert(
        Did(1),
        crate::IdtyValue {
            index: 0,
            owner_key: 1,
            removable_on: None,
            rights: vec![(Right::Right2, Some(10))],
            status: crate::IdtyStatus::Validated,
            data: (),
        },
    );
    identities.insert(
        Did(2),
        crate::IdtyValue {
            index: 1,
            owner_key: 2,
            removable_on: None,
            rights: vec![(Right::Right1, Some(20))],
            status: crate::IdtyStatus::Validated,
            data: (),
        },
    );
    new_test_ext(IdentityConfig { identities }).execute_with(|| {
        // Should have two identities
        assert_eq!(Identity::identities_count(), 2);

        // We need to initialize at least one block before any call
        run_to_block(1);

        // Add right Right1 for Did(1)
        // Should succes and trigger the correct event
        assert_ok!(Identity::add_right(Origin::root(), Did(1), Right::Right1));
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyAcquireRight(Did(1), Right::Right1)),
                topics: vec![],
            }
        );
        // Add right Right2 for Did(1)
        // Should fail because Did(1) already have this right
        assert_err!(
            Identity::add_right(Origin::root(), Did(1), Right::Right2),
            Error::<Test>::RightAlreadyAdded
        );

        run_to_block(3);

        // Delete right Right1 for Did(2)
        // Should succes and trigger the correct event
        assert_ok!(Identity::del_right(Origin::root(), Did(2), Right::Right1));
        let events = System::events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[1],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::Identity(crate::Event::IdtyLostRight(Did(2), Right::Right1)),
                topics: vec![],
            }
        );

        // The Did(2) identity has no more rights, the inactivity period must start to run
        let idty2 = Identity::identity(Did(2));
        assert!(idty2.rights.is_empty());
        assert_eq!(idty2.removable_on, Some(7));
    });
}
