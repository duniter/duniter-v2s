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
use frame_system::{EventRecord, Phase};

#[test]
fn test_ud_creation() {
    new_test_ext(UniversalDividendConfig {
        first_ud: 1_000,
        initial_monetary_mass: 0,
    })
    .execute_with(|| {
        // In the beginning there was no money
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 0);

        // The first UD must be created in block #2
        run_to_block(2);
        assert_eq!(Balances::free_balance(1), 1_000);
        assert_eq!(Balances::free_balance(2), 1_000);
        assert_eq!(Balances::free_balance(3), 1_000);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 3_000);

        // Block #2 must generate 7 events, 2 events per new account fed, plus 1 event for the creation of the UD.
        let events = System::events();
        assert_eq!(events.len(), 7);
        assert_eq!(
            events[6],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::UniversalDividend(crate::Event::NewUdCreated(1000, 3)),
                topics: vec![],
            }
        );

        // The second UD must be created in block #4
        run_to_block(4);
        assert_eq!(Balances::free_balance(1), 2_000);
        assert_eq!(Balances::free_balance(2), 2_000);
        assert_eq!(Balances::free_balance(3), 2_000);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 6_000);

        /*// Block #4 must generate 4 events, 1 event per account fed, plus 1 event for the creation of the UD.
        let events = System::events();
        println!("{:?}", events);
        assert_eq!(events.len(), 4);
        assert_eq!(
            events[3],
            EventRecord {
                phase: Phase::Initialization,
                event: Event::UniversalDividend(crate::Event::NewUdCreated(1000, 3)),
                topics: vec![],
            }
        );*/

        // The third UD must be created in block #6
        run_to_block(6);
        assert_eq!(Balances::free_balance(1), 3_000);
        assert_eq!(Balances::free_balance(2), 3_000);
        assert_eq!(Balances::free_balance(3), 3_000);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 9_000);

        // Block #8 should cause a re-evaluation of UD
        run_to_block(8);
        assert_eq!(Balances::free_balance(1), 4_025);
        assert_eq!(Balances::free_balance(2), 4_025);
        assert_eq!(Balances::free_balance(3), 4_025);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 12_075);

        // Block #10 #12 and #14should creates the reevalued UD
        run_to_block(14);
        assert_eq!(Balances::free_balance(1), 7_100);
        assert_eq!(Balances::free_balance(2), 7_100);
        assert_eq!(Balances::free_balance(3), 7_100);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 21_300);

        // Block #16 should cause a second re-evaluation of UD
        run_to_block(16);
        assert_eq!(Balances::free_balance(1), 8_200);
        assert_eq!(Balances::free_balance(2), 8_200);
        assert_eq!(Balances::free_balance(3), 8_200);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 24_600);
    });
}
