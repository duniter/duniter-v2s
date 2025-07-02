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

use crate::mock::*;
use frame_support::{assert_err, assert_ok, assert_storage_noop, traits::ReservableCurrency};
use sp_runtime::{ArithmeticError, DispatchError};

#[test]
fn test_claim_uds() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // In the beginning there was no money
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 0);

        // Alice can claim UDs, but this should be a no-op.
        run_to_block(1);
        assert_storage_noop!(assert_ok!(UniversalDividend::claim_uds(
            RuntimeOrigin::signed(1)
        )));
        assert_eq!(Balances::free_balance(1), 0);

        // Dave is not a member, he can't claim UDs
        assert_err!(
            UniversalDividend::claim_uds(RuntimeOrigin::signed(4)),
            crate::Error::<Test>::AccountNotAllowedToClaimUds
        );

        // At block #2, the first UD must be created, but nobody should receive money
        run_to_block(2);
        assert_eq!(UniversalDividend::total_money_created(), 3_000);
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Alice can claim UDs, and this time she must receive exactly one UD
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(1)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 1,
            total: 1_000,
            who: 1,
        }));
        // the expected event from pallet balances is Minted
        System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Minted {
            who: 1,
            amount: 1000,
        }));
        assert_eq!(Balances::free_balance(1), 1_000);
        // Others members should not receive any UDs with Alice claim
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // At block #4, the second UD must be created, but nobody should receive money
        run_to_block(4);
        assert_eq!(UniversalDividend::total_money_created(), 6_000);
        assert_eq!(Balances::free_balance(1), 1_000);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Alice can claim UDs, And she must receive exactly one UD (the second one)
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(1)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 1,
            total: 1_000,
            who: 1,
        }));
        assert_eq!(Balances::free_balance(1), 2_000);
        // Others members should not receive any UDs with Alice claim
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Bob can claim UDs, he must receive exactly two UDs
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(2)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 2,
            total: 2_000,
            who: 2,
        }));
        assert_eq!(Balances::free_balance(2), 2_000);
        // Others members should not receive any UDs with Alice and Bob claims
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Dave is still not a member, he still can't claim UDs.
        assert_err!(
            UniversalDividend::claim_uds(RuntimeOrigin::signed(4)),
            crate::Error::<Test>::AccountNotAllowedToClaimUds
        );

        // At block #8, the first reevaluated UD should be created
        run_to_block(8);
        assert_eq!(UniversalDividend::total_money_created(), 12_225);

        // Charlie can claim all his UDs at once, he must receive exactly four UDs
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(3)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 4,
            total: 4_075,
            who: 3,
        }));
        assert_eq!(Balances::free_balance(3), 4_075);

        // At block #16, the second reevaluated UD should be created
        run_to_block(16);
        assert_eq!(UniversalDividend::total_money_created(), 25_671);

        // Charlie can claim new UD, he must receive exactly four UDs
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(3)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 4,
            total: 4_482,
            who: 3,
        }));
        assert_eq!(Balances::free_balance(3), 8557);
    });
}

#[test]
fn test_claim_uds_using_genesis_timestamp() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: None,
        first_ud: None,
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // In the beginning there was no money
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);
        assert_eq!(UniversalDividend::total_money_created(), 0);

        // Alice can claim UDs, but this should be a no-op.
        run_to_block(1);
        assert_storage_noop!(assert_ok!(UniversalDividend::claim_uds(
            RuntimeOrigin::signed(1)
        )));
        assert_eq!(Balances::free_balance(1), 0);

        // Dave is not a member, he can't claim UDs
        assert_err!(
            UniversalDividend::claim_uds(RuntimeOrigin::signed(4)),
            crate::Error::<Test>::AccountNotAllowedToClaimUds
        );

        // At block #3, the first UD must be created, but nobody should receive money
        run_to_block(3);
        assert_eq!(UniversalDividend::total_money_created(), 3_000);
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Alice can claim UDs, and this time she must receive exactly one UD
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(1)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 1,
            total: 1_000,
            who: 1,
        }));
        assert_eq!(Balances::free_balance(1), 1_000);
        // Others members should not receive any UDs with Alice claim
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // At block #5, the second UD must be created, but nobody should receive money
        run_to_block(5);
        assert_eq!(UniversalDividend::total_money_created(), 6_000);
        assert_eq!(Balances::free_balance(1), 1_000);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Alice can claim UDs, And she must receive exactly one UD (the second one)
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(1)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 1,
            total: 1_000,
            who: 1,
        }));
        assert_eq!(Balances::free_balance(1), 2_000);
        // Others members should not receive any UDs with Alice claim
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Bob can claim UDs, he must receive exactly two UDs
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(2)));
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdsClaimed {
            count: 2,
            total: 2_000,
            who: 2,
        }));
        assert_eq!(Balances::free_balance(2), 2_000);
        // Others members should not receive any UDs with Alice and Bob claims
        assert_eq!(Balances::free_balance(3), 0);
        assert_eq!(Balances::free_balance(4), 0);

        // Dave is still not a member, he still can't claim UDs.
        assert_err!(
            UniversalDividend::claim_uds(RuntimeOrigin::signed(4)),
            crate::Error::<Test>::AccountNotAllowedToClaimUds
        );

        // At block #11, the first reevaluated UD should be created
        run_to_block(11);
        assert_eq!(UniversalDividend::total_money_created(), 15_300);
    });
}

#[test]
fn test_ud_creation() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
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
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            crate::Event::NewUdCreated {
                amount: 1_000,
                index: 1,
                monetary_mass: 3_000,
                members_count: 3,
            },
        ));
        assert_eq!(UniversalDividend::total_money_created(), 3_000);
        /*assert_eq!(Balances::free_balance(1), 1_000);
        assert_eq!(Balances::free_balance(2), 1_000);
        assert_eq!(Balances::free_balance(3), 1_000);
        assert_eq!(Balances::free_balance(4), 0);*/

        // The second UD must be created in block #4
        run_to_block(4);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            crate::Event::NewUdCreated {
                amount: 1_000,
                index: 2,
                monetary_mass: 6_000,
                members_count: 3,
            },
        ));
        assert_eq!(UniversalDividend::total_money_created(), 6_000);
        /*assert_eq!(Balances::free_balance(1), 2_000);
        assert_eq!(Balances::free_balance(2), 2_000);
        assert_eq!(Balances::free_balance(3), 2_000);
        assert_eq!(Balances::free_balance(4), 0);*/

        // The third UD must be created in block #6
        run_to_block(6);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            crate::Event::NewUdCreated {
                amount: 1_000,
                index: 3,
                monetary_mass: 9_000,
                members_count: 3,
            },
        ));
        assert_eq!(UniversalDividend::total_money_created(), 9_000);
        /*assert_eq!(Balances::free_balance(1), 3_000);
        assert_eq!(Balances::free_balance(2), 3_000);
        assert_eq!(Balances::free_balance(3), 3_000);
        assert_eq!(Balances::free_balance(4), 0);*/

        // Block #8 should cause a re-evaluation of UD
        run_to_block(8);
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdReevalued {
            new_ud_amount: 1_075,
            monetary_mass: 9_000,
            members_count: 3,
        }));
        // Then, the first reevalued UD should be created
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            crate::Event::NewUdCreated {
                amount: 1_075,
                index: 4,
                monetary_mass: 12_225,
                members_count: 3,
            },
        ));
        assert_eq!(UniversalDividend::total_money_created(), 12_225);
        /*assert_eq!(Balances::free_balance(1), 4_075);
        assert_eq!(Balances::free_balance(2), 4_075);
        assert_eq!(Balances::free_balance(3), 4_075);
        assert_eq!(Balances::free_balance(4), 0);*/

        // Block #10 #12 and #14should creates the reevalued UD
        run_to_block(14);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            crate::Event::NewUdCreated {
                amount: 1_075,
                index: 7,
                monetary_mass: 21_900,
                members_count: 3,
            },
        ));
        assert_eq!(UniversalDividend::total_money_created(), 21_900);

        // Block #16 should cause a second re-evaluation of UD
        run_to_block(16);
        System::assert_has_event(RuntimeEvent::UniversalDividend(crate::Event::UdReevalued {
            new_ud_amount: 1_257,
            monetary_mass: 21_900,
            members_count: 3,
        }));
        // Then, the reevalued UD should be created
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            crate::Event::NewUdCreated {
                amount: 1_257,
                index: 8,
                monetary_mass: 25_671,
                members_count: 3,
            },
        ));
        assert_eq!(UniversalDividend::total_money_created(), 25_671);
    });
}

#[test]
fn test_account_balances() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Initially, all accounts have zero balance
        let balance_info = UniversalDividend::account_balances(&1);
        assert_eq!(balance_info.transferable, 0);
        assert_eq!(balance_info.total, 0);
        assert_eq!(balance_info.unclaim_uds, 0);

        // Create some UDs and claim them
        run_to_block(2);
        assert_ok!(UniversalDividend::claim_uds(RuntimeOrigin::signed(1)));

        // Check balance after claiming
        let balance_info = UniversalDividend::account_balances(&1);
        assert_eq!(balance_info.transferable, 1000 - 10); // free (1000) + unclaim_uds (0) - existantial deposit (10)
        assert_eq!(balance_info.total, 1000); // transferable + reserved
        assert_eq!(balance_info.unclaim_uds, 0);

        // Create more UDs but don't claim them
        run_to_block(4);
        let balance_info = UniversalDividend::account_balances(&1);
        assert_eq!(balance_info.transferable, 1000 + 1000 - 10); // free (1000) + unclaim_uds (1000) - existantial deposit (10)
        assert_eq!(balance_info.total, 2000); // transferable + reserved
        assert_eq!(balance_info.unclaim_uds, 1000);

        // Test with reserved balance
        assert_ok!(Balances::reserve(&1, 500));
        let balance_info = UniversalDividend::account_balances(&1);
        assert_eq!(balance_info.transferable, 500 + 1000 - 10); // free (500) + unclaim_uds (1000) - existantial deposit (10)
        assert_eq!(balance_info.total, 2000); // transferable + reserved
        assert_eq!(balance_info.unclaim_uds, 1000);

        // Test non-member account
        let balance_info = UniversalDividend::account_balances(&4);
        assert_eq!(balance_info.transferable, 0);
        assert_eq!(balance_info.total, 0);
        assert_eq!(balance_info.unclaim_uds, 0);
    });
}

#[test]
fn test_transfer_ud_overflow() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 some balance to work with
        let _ = mint_into(&1, 1_000_000);
        assert_eq!(Balances::free_balance(1), 1_000_000);

        // Test overflow scenario: try to transfer a very large value in milliUD
        // that when multiplied by current_ud (1000) would overflow u64
        let max_u64 = u64::MAX;
        let overflow_value = max_u64 / 1000 + 1; // This will overflow when multiplied by 1000

        assert_err!(
            UniversalDividend::transfer_ud(RuntimeOrigin::signed(1), 2, overflow_value),
            DispatchError::Arithmetic(ArithmeticError::Overflow),
        );
    });
}

#[test]
fn test_transfer_ud_keep_alive_overflow() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 some balance to work with
        let _ = mint_into(&1, 1_000_000);
        assert_eq!(Balances::free_balance(1), 1_000_000);

        // Test overflow scenario: try to transfer a very large value in milliUD
        // that when multiplied by current_ud (1000) would overflow u64
        let max_u64 = u64::MAX;
        let overflow_value = max_u64 / 1000 + 1; // This will overflow when multiplied by 1000

        assert_err!(
            UniversalDividend::transfer_ud_keep_alive(RuntimeOrigin::signed(1), 2, overflow_value),
            DispatchError::Arithmetic(ArithmeticError::Overflow),
        );
    });
}

#[test]
fn test_transfer_ud_underflow() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 some balance to work with
        let _ = mint_into(&1, 1_000_000);
        assert_eq!(Balances::free_balance(1), 1_000_000);

        // Test underflow scenario: try to transfer a value that when divided by 1000
        // would result in 0 (which is not a valid transfer amount)
        let underflow_value = 999; // 999 * 1000 / 1000 = 999, which is valid

        // This should work because 999 milliUD = 999 actual units
        assert_ok!(UniversalDividend::transfer_ud(
            RuntimeOrigin::signed(1),
            2,
            underflow_value
        ));
        assert_eq!(Balances::free_balance(2), 999);

        // Test with minimum valid value (1000 milliUD = 1 UD)
        assert_ok!(UniversalDividend::transfer_ud(
            RuntimeOrigin::signed(1),
            2,
            1000
        ));
        assert_eq!(Balances::free_balance(2), 1999); // 999 + 1000
    });
}

#[test]
fn test_transfer_ud_keep_alive_underflow() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 some balance to work with
        let _ = mint_into(&1, 1_000_000);
        assert_eq!(Balances::free_balance(1), 1_000_000);

        // Test underflow scenario: try to transfer a value that when divided by 1000
        // would result in 0 (which is not a valid transfer amount)
        let underflow_value = 999; // 999 * 1000 / 1000 = 999, which is valid

        // This should work because 999 milliUD = 999 actual units
        assert_ok!(UniversalDividend::transfer_ud_keep_alive(
            RuntimeOrigin::signed(1),
            2,
            underflow_value
        ));
        assert_eq!(Balances::free_balance(2), 999);

        // Test with minimum valid value (1000 milliUD = 1 UD)
        assert_ok!(UniversalDividend::transfer_ud_keep_alive(
            RuntimeOrigin::signed(1),
            2,
            1000
        ));
        assert_eq!(Balances::free_balance(2), 1999); // 999 + 1000
    });
}

#[test]
fn test_transfer_ud_edge_cases() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 some balance to work with
        let _ = mint_into(&1, 1_000_000);
        assert_eq!(Balances::free_balance(1), 1_000_000);

        // Test with zero value (should work - 0 * 1000 / 1000 = 0)
        assert_ok!(UniversalDividend::transfer_ud(
            RuntimeOrigin::signed(1),
            2,
            0
        ));
        assert_eq!(Balances::free_balance(2), 0);

        // Test with very small values that should work
        assert_ok!(UniversalDividend::transfer_ud(
            RuntimeOrigin::signed(1),
            2,
            1000
        )); // 1 UD
        assert_eq!(Balances::free_balance(2), 1000);

        assert_ok!(UniversalDividend::transfer_ud(
            RuntimeOrigin::signed(1),
            2,
            1500
        )); // 1.5 UD
        assert_eq!(Balances::free_balance(2), 2500);
    });
}

#[test]
fn test_transfer_ud_keep_alive_edge_cases() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 some balance to work with
        let _ = mint_into(&1, 1_000_000);
        assert_eq!(Balances::free_balance(1), 1_000_000);

        // Test with zero value (should work - 0 * 1000 / 1000 = 0)
        assert_ok!(UniversalDividend::transfer_ud_keep_alive(
            RuntimeOrigin::signed(1),
            2,
            0
        ));
        assert_eq!(Balances::free_balance(2), 0);

        // Test with very small values that should work
        assert_ok!(UniversalDividend::transfer_ud_keep_alive(
            RuntimeOrigin::signed(1),
            2,
            1000
        )); // 1 UD
        assert_eq!(Balances::free_balance(2), 1000);

        assert_ok!(UniversalDividend::transfer_ud_keep_alive(
            RuntimeOrigin::signed(1),
            2,
            1500
        )); // 1.5 UD
        assert_eq!(Balances::free_balance(2), 2500);
    });
}

#[test]
fn test_transfer_ud_insufficient_balance() {
    new_test_ext(UniversalDividendConfig {
        first_reeval: Some(48_000),
        first_ud: Some(12_000),
        initial_monetary_mass: 0,
        initial_members: vec![1, 2, 3],
        ud: 1_000,
    })
    .execute_with(|| {
        // Give account 1 minimal balance
        let _ = mint_into(&1, 100);
        assert_eq!(Balances::free_balance(1), 100);

        // Try to transfer more than available balance
        assert_err!(
            UniversalDividend::transfer_ud(RuntimeOrigin::signed(1), 2, 2000), // Would require 2000 balance
            DispatchError::Arithmetic(ArithmeticError::Underflow),
        );

        // Try to transfer exactly the available balance
        assert_ok!(UniversalDividend::transfer_ud(
            RuntimeOrigin::signed(1),
            2,
            100
        )); // 100 milliUD = 0.1 UD
        assert_eq!(Balances::free_balance(2), 100);
        assert_eq!(Balances::free_balance(1), 0);
    });
}
