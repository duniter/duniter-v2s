// Copyright 2023 Axiom-Team
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
use frame_support::{assert_noop, assert_ok, traits::fungible::Mutate};

// allow request distance evaluation for oneself
#[test]
fn test_request_distance_evaluation() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        // give enough for reserve
        Balances::set_balance(&1, 10_000);

        // call request
        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        System::assert_has_event(RuntimeEvent::Distance(Event::EvaluationRequested {
            idty_index: 1,
            who: 1,
        }));

        // currency was reserved
        assert_eq!(Balances::reserved_balance(1), 1000);
    });
}

// allow request distance evaluation for an unvalidated identity
#[test]
fn test_request_distance_evaluation_for() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        // give enough for reserve
        Balances::set_balance(&1, 10_000);
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 5));
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(5),
            "Eeeve".into()
        ));

        // call request
        assert_ok!(Distance::request_distance_evaluation_for(
            RuntimeOrigin::signed(1),
            5
        ));
        System::assert_has_event(RuntimeEvent::Distance(Event::EvaluationRequested {
            idty_index: 5,
            who: 1,
        }));

        // currency was reserved
        assert_eq!(Balances::reserved_balance(1), 1000);

        // let the request expire
        run_to_block(12);
        System::assert_has_event(RuntimeEvent::Distance(Event::NotEvaluated {
            idty_index: 5,
            who: 1,
        }));
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::free_balance(1), 10_000);
    });
}

// non member can not request distance evaluation
#[test]
fn test_request_distance_evaluation_non_member() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        // give enough for reserve
        Balances::set_balance(&5, 10_000);

        assert_noop!(
            Distance::request_distance_evaluation_for(RuntimeOrigin::signed(5), 1),
            Error::<Test>::CallerHasNoIdentity
        );
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 5));
        assert_noop!(
            Distance::request_distance_evaluation_for(RuntimeOrigin::signed(5), 1),
            Error::<Test>::CallerNotMember
        );
    });
}

// can not request distance eval if already in evaluation
#[test]
fn test_request_distance_evaluation_twice() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        // give enough for reserve
        Balances::set_balance(&1, 10_000);

        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        assert_noop!(
            Distance::request_distance_evaluation(RuntimeOrigin::signed(1)),
            Error::<Test>::AlreadyInEvaluation
        );
    });
}
