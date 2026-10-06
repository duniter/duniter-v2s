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
        // ensure account exists before creating an identity for it
        Balances::set_balance(&5, 10_000);
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

fn submit_result(distance: sp_runtime::Perbill) {
    run_to_block(8);
    assert_ok!(Distance::force_update_evaluation(
        RuntimeOrigin::root(),
        1,
        ComputationResult {
            distances: vec![distance]
        },
    ));
    run_to_block(12);
}

#[test]
fn negative_result_refunds_and_delays_self_requests() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        Balances::set_balance(&1, 10_000);
        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        submit_result(sp_runtime::Perbill::from_percent(79));
        assert_eq!(Balances::free_balance(1), 10_000);
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(NextEvaluationOn::<Test>::get(1), Some(20));
        assert_eq!(PendingEvaluationRequest::<Test>::get(1), None);
        System::assert_has_event(RuntimeEvent::Distance(Event::EvaluatedInvalid {
            idty_index: 1,
            distance: sp_runtime::Perbill::from_percent(79),
        }));
        run_to_block(19);
        assert_noop!(
            Distance::request_distance_evaluation(RuntimeOrigin::signed(1)),
            Error::<Test>::DistanceRetryPeriodNotRespected
        );
        run_to_block(20);
        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        assert_eq!(NextEvaluationOn::<Test>::get(1), None);
    });
}

#[test]
fn retry_delay_follows_target_across_requesters_and_owner_changes() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        Balances::set_balance(&1, 10_000);
        Balances::set_balance(&2, 10_000);
        Balances::set_balance(&5, 10_000);
        assert_ok!(Identity::create_identity(RuntimeOrigin::signed(1), 5));
        assert_ok!(Identity::confirm_identity(
            RuntimeOrigin::signed(5),
            "Eeeve".into()
        ));
        assert_ok!(Distance::request_distance_evaluation_for(
            RuntimeOrigin::signed(1),
            5
        ));
        submit_result(sp_runtime::Perbill::zero());
        assert_eq!(Balances::free_balance(1), 10_000);
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_noop!(
            Distance::request_distance_evaluation_for(RuntimeOrigin::signed(2), 5),
            Error::<Test>::DistanceRetryPeriodNotRespected
        );
        assert_noop!(
            Distance::request_distance_evaluation(RuntimeOrigin::signed(5)),
            Error::<Test>::DistanceRetryPeriodNotRespected
        );
        // Model an owner-key change without changing the identity index.
        pallet_identity::IdentityIndexOf::<Test>::remove(5);
        pallet_identity::IdentityIndexOf::<Test>::insert(6, 5);
        pallet_identity::Identities::<Test>::mutate(5, |idty| idty.as_mut().unwrap().owner_key = 6);
        assert_noop!(
            Distance::request_distance_evaluation(RuntimeOrigin::signed(6)),
            Error::<Test>::DistanceRetryPeriodNotRespected
        );
        run_to_block(20);
        assert_ok!(Distance::request_distance_evaluation_for(
            RuntimeOrigin::signed(2),
            5
        ));
        assert_eq!(NextEvaluationOn::<Test>::get(5), None);
    });
}

#[test]
fn successful_and_missing_results_do_not_delay_retries() {
    for result in [Some(sp_runtime::Perbill::from_percent(80)), None] {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            Balances::set_balance(&1, 10_000);
            assert_ok!(Distance::request_distance_evaluation(
                RuntimeOrigin::signed(1)
            ));
            if let Some(distance) = result {
                submit_result(distance);
            } else {
                run_to_block(12);
                System::assert_has_event(RuntimeEvent::Distance(Event::NotEvaluated {
                    idty_index: 1,
                    who: 1,
                }));
            }
            assert_eq!(NextEvaluationOn::<Test>::get(1), None);
            assert_eq!(Balances::free_balance(1), 10_000);
            assert_eq!(Balances::reserved_balance(1), 0);
            assert_ok!(Distance::request_distance_evaluation(
                RuntimeOrigin::signed(1)
            ));
        });
    }
}

#[test]
fn retry_delay_uses_configured_period() {
    new_test_ext().execute_with(|| {
        DistanceRetryPeriod::set(20);
        run_to_block(1);
        Balances::set_balance(&1, 10_000);
        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        submit_result(sp_runtime::Perbill::zero());
        assert_eq!(NextEvaluationOn::<Test>::get(1), Some(32));
        run_to_block(31);
        assert_noop!(
            Distance::request_distance_evaluation(RuntimeOrigin::signed(1)),
            Error::<Test>::DistanceRetryPeriodNotRespected
        );
        run_to_block(32);
        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        DistanceRetryPeriod::set(8);
    });
}

#[test]
fn removed_identity_is_refunded_without_storing_a_delay() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        Balances::set_balance(&1, 10_000);
        assert_ok!(Distance::request_distance_evaluation(
            RuntimeOrigin::signed(1)
        ));
        pallet_identity::Identities::<Test>::remove(1);
        submit_result(sp_runtime::Perbill::zero());
        assert_eq!(Balances::free_balance(1), 10_000);
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(NextEvaluationOn::<Test>::get(1), None);
    });
}

#[test]
fn identity_removal_cleans_retry_state() {
    new_test_ext().execute_with(|| {
        NextEvaluationOn::<Test>::insert(1, 100);
        Identity::do_remove_identity(1, pallet_identity::RemovalReason::Revoked);
        assert_eq!(NextEvaluationOn::<Test>::get(1), None);
    });
}

// Measure the complete bounded result-processing route at the production pool limits.
#[test]
fn full_negative_pool_load() {
    use frame_support::traits::fungible::MutateHold;
    use std::time::Instant;

    let mut timings = Vec::new();
    for _ in 0..10 {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            Balances::set_balance(&1, 10_000_000);
            let identity = pallet_identity::Identities::<Test>::get(1).unwrap();
            let mut pool = EvaluationPool::<u64, u32>::default();
            for idty in 1..=MAX_EVALUATIONS_PER_SESSION {
                pallet_identity::Identities::<Test>::insert(idty, identity.clone());
                PendingEvaluationRequest::<Test>::insert(idty, 1);
                Balances::hold(&HoldReason::DistanceHold.into(), &1, 1000).unwrap();
                let mut median = median::MedianAcc::new();
                for _ in 0..MAX_EVALUATORS_PER_SESSION {
                    median.push(sp_runtime::Perbill::from_percent(79));
                }
                pool.evaluations.try_push((idty, median)).unwrap();
            }
            EvaluationPool2::<Test>::put(pool);
            let start = Instant::now();
            let weight = Distance::do_evaluation(0);
            timings.push(start.elapsed());
            assert!(weight.ref_time() > 0);
            assert_eq!(
                NextEvaluationOn::<Test>::iter().count(),
                MAX_EVALUATIONS_PER_SESSION as usize
            );
            assert_eq!(PendingEvaluationRequest::<Test>::iter().count(), 0);
            assert_eq!(Balances::reserved_balance(1), 0);
            assert_eq!(Balances::free_balance(1), 10_000_000);
            assert!(EvaluationPool2::<Test>::get().evaluations.is_empty());
        });
    }
    timings.sort();
    println!(
        "Full negative pool: {} identities, {} evaluators, 10 samples, median {:?}, max {:?}",
        MAX_EVALUATIONS_PER_SESSION, MAX_EVALUATORS_PER_SESSION, timings[5], timings[9]
    );
}
