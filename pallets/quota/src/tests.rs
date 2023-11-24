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
use crate::Weight;
use frame_support::traits::Currency;
use sp_core::Get;

// Note: values for reload rate and max quota defined in mock file
// parameter_types! {
//     pub const ReloadRate: u64 = 10;
//     pub const MaxQuota: u64 = 1000;
// }
// pub const ExistentialDeposit: Balance = 1000;

/// test that quota are well initialized for genesis identities
#[test]
fn test_initial_quota() {
    new_test_ext(QuotaConfig {
        identities: vec![1, 2, 3],
    })
    .execute_with(|| {
        run_to_block(1);

        // quota initialized to 0,0 for a given identity
        assert_eq!(
            Quota::quota(1),
            Some(pallet_quota::Quota {
                last_use: 0,
                amount: 0
            })
        );
        // no initialized quota for standard account
        assert_eq!(Quota::quota(4), None);
    })
}

/// test that quota are updated according to the reload rate and max quota values
#[test]
fn test_update_quota() {
    new_test_ext(QuotaConfig {
        identities: vec![1, 2, 3],
    })
    .execute_with(|| {
        // Block 1
        run_to_block(1);
        assert_eq!(
            Quota::quota(1),
            Some(pallet_quota::Quota {
                last_use: 0,
                amount: 0
            })
        );
        // (spending 0 quota will lead to only update)
        // assert zero quota spent
        assert_eq!(Quota::spend_quota(1, 0), 0);
        assert_eq!(
            Quota::quota(1),
            Some(pallet_quota::Quota {
                last_use: 1, // used at block 1
                // max quota × (current block - last use) / reload rate
                amount: 100 // 1000 × 1 / 10 = 100
            })
        );

        // Block 2
        run_to_block(2);
        assert_eq!(Quota::spend_quota(2, 0), 0);
        assert_eq!(
            Quota::quota(2),
            Some(pallet_quota::Quota {
                last_use: 2, // used at block 2
                // max quota × (current block - last use) / reload rate
                amount: 200 // 1000 × 2 / 10 = 200
            })
        );

        // Block 20
        run_to_block(20);
        assert_eq!(Quota::spend_quota(2, 0), 0);
        assert_eq!(
            Quota::quota(2),
            Some(pallet_quota::Quota {
                last_use: 20, // used at block 20
                // maximum quota is reached
                // 1000 × (20 - 2) / 10 = 1800
                amount: 1000 // min(1000, 1800)
            })
        );
    })
}

/// test that right amount of quota is spent
#[test]
fn test_spend_quota() {
    new_test_ext(QuotaConfig {
        identities: vec![1, 2, 3],
    })
    .execute_with(|| {
        // at block 5, quota are half loaded (500)
        run_to_block(5);
        // spending less than available
        assert_eq!(Quota::spend_quota(1, 200), 200);
        assert_eq!(
            Quota::quota(1),
            Some(pallet_quota::Quota {
                last_use: 5,
                amount: 300 // 500 - 200
            })
        );
        // spending all available
        assert_eq!(Quota::spend_quota(2, 500), 500);
        assert_eq!(
            Quota::quota(2),
            Some(pallet_quota::Quota {
                last_use: 5,
                amount: 0 // 500 - 500
            })
        );
        // spending more than available
        assert_eq!(Quota::spend_quota(3, 1000), 500);
        assert_eq!(
            Quota::quota(3),
            Some(pallet_quota::Quota {
                last_use: 5,
                amount: 0 // 500 - 500
            })
        );
    })
}

/// test complete scenario with queue and process refund queue
#[test]
fn test_process_refund_queue() {
    new_test_ext(QuotaConfig {
        identities: vec![1, 2],
    })
    .execute_with(|| {
        run_to_block(5);
        // give enough currency to accounts and treasury and double check
        Balances::make_free_balance_be(&account(1), 1000);
        Balances::make_free_balance_be(&account(2), 1000);
        Balances::make_free_balance_be(&account(3), 1000);
        Balances::make_free_balance_be(
            &<Test as pallet_quota::Config>::RefundAccount::get(),
            10_000,
        );
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            10_000
        );
        // fill in the refund queue
        Quota::queue_refund(pallet_quota::Refund {
            account: account(1),
            identity: 1,
            amount: 10,
        });
        Quota::queue_refund(pallet_quota::Refund {
            account: account(2),
            identity: 2,
            amount: 1000,
        });
        Quota::queue_refund(pallet_quota::Refund {
            account: account(3),
            identity: 3,
            amount: 666,
        });
        // process it
        Quota::process_refund_queue(Weight::from(10));
        // after processing, it should be empty
        assert!(pallet_quota::RefundQueue::<Test>::get().is_empty());
        // and we should observe the effects of refund
        assert_eq!(Balances::free_balance(account(1)), 1010); // 1000 initial + 10 refunded
        assert_eq!(Balances::free_balance(account(2)), 1500); // 1000 initial + 500 refunded
        assert_eq!(Balances::free_balance(account(3)), 1000); // only initial because no available quota
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            // initial minus refunds
            10_000 - 500 - 10
        );
        // events
        System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
            who: account(1),
            identity: 1,
            amount: 10,
        }));
        System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::NoQuotaForIdty(3)));
    })
}

/// test not enough currency in treasury
#[test]
fn test_not_enough_treasury() {
    new_test_ext(QuotaConfig {
        identities: vec![1],
    })
    .execute_with(|| {
        run_to_block(5);
        Balances::make_free_balance_be(&account(1), 1000);
        Balances::make_free_balance_be(&<Test as pallet_quota::Config>::RefundAccount::get(), 1200);
        Quota::queue_refund(pallet_quota::Refund {
            account: account(1),
            identity: 1,
            amount: 500,
        });
        Quota::process_refund_queue(Weight::from(10));
        // refund was not possible, would kill treasury
        assert_eq!(Balances::free_balance(account(1)), 1000);
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            1200
        );
        // event
        System::assert_has_event(RuntimeEvent::Quota(
            pallet_quota::Event::NoMoreCurrencyForRefund,
        ));
        // quotas were spent anyway, there is no refund for quotas when refund account is empty
        assert_eq!(
            Quota::quota(1),
            Some(pallet_quota::Quota {
                last_use: 5,
                amount: 0
            })
        );
    })
}

/// test complete scenario with queue and process refund queue weight with available quotas
#[test]
fn test_process_refund_queue_weight_with_quotas() {
    new_test_ext(QuotaConfig {
        identities: vec![1, 2, 3],
    })
    .execute_with(|| {
        run_to_block(15);
        // give enough currency to accounts and treasury and double check
        Balances::make_free_balance_be(&account(1), 1000);
        Balances::make_free_balance_be(&account(2), 1000);
        Balances::make_free_balance_be(&account(3), 1000);
        Balances::make_free_balance_be(
            &<Test as pallet_quota::Config>::RefundAccount::get(),
            10_000,
        );
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            10_000
        );
        // fill in the refund queue
        Quota::queue_refund(pallet_quota::Refund {
            account: account(1),
            identity: 10,
            amount: 10,
        });
        Quota::queue_refund(pallet_quota::Refund {
            account: account(2),
            identity: 2,
            amount: 500,
        });
        Quota::queue_refund(pallet_quota::Refund {
            account: account(3),
            identity: 3,
            amount: 666,
        });
        // process it with only no weight
        Quota::process_refund_queue(Weight::from(0));
        // after processing, it should be of the same size
        assert_eq!(pallet_quota::RefundQueue::<Test>::get().len(), 3);
        // process it with only 200 allowed weight
        Quota::process_refund_queue(Weight::from_parts(200u64, 0));
        // after processing, it should be of size 1 because total_weight += 25*2 by iteration and
        // limit is total_weight < 200-100 so 2 elements can be processed
        assert_eq!(pallet_quota::RefundQueue::<Test>::get().len(), 1);
        // and we should observe the effects of refund
        assert_eq!(Balances::free_balance(account(3)), 1666); // 1000 initial + 666 refunded
        assert_eq!(Balances::free_balance(account(2)), 1500); // 1000 initial + 1500 refunded
        assert_eq!(Balances::free_balance(account(1)), 1000); // only initial because no available weight to process
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            // initial minus refunds
            10_000 - 666 - 500
        );
        // events
        System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
            who: account(3),
            identity: 3,
            amount: 666,
        }));
        System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
            who: account(2),
            identity: 2,
            amount: 500,
        }));
    })
}

/// test complete scenario with queue and process refund queue weight with limited quotas
#[test]
fn test_process_refund_queue_weight_no_quotas() {
    new_test_ext(QuotaConfig {
        identities: vec![1, 2],
    })
    .execute_with(|| {
        run_to_block(15);
        // give enough currency to accounts and treasury and double check
        Balances::make_free_balance_be(&account(1), 1000);
        Balances::make_free_balance_be(&account(2), 1000);
        Balances::make_free_balance_be(&account(3), 1000);
        Balances::make_free_balance_be(
            &<Test as pallet_quota::Config>::RefundAccount::get(),
            10_000,
        );
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            10_000
        );
        // fill in the refund queue
        Quota::queue_refund(pallet_quota::Refund {
            account: account(1),
            identity: 10,
            amount: 10,
        });
        Quota::queue_refund(pallet_quota::Refund {
            account: account(2),
            identity: 2,
            amount: 500,
        });
        Quota::queue_refund(pallet_quota::Refund {
            account: account(3),
            identity: 3,
            amount: 666,
        });
        // process it with only no weight
        Quota::process_refund_queue(Weight::from(0));
        // after processing, it should be of the same size
        assert_eq!(pallet_quota::RefundQueue::<Test>::get().len(), 3);
        // process it with only 150 allowed weight
        Quota::process_refund_queue(Weight::from_parts(150u64, 0));
        // after processing, it should be of size 2 because try_refund weight is 25 (first in the queue with no quota) then 25*2 for the 2 other elements
        // limit is total_weight < 150-100 so 2 elements can be processed
        assert_eq!(pallet_quota::RefundQueue::<Test>::get().len(), 1);
        // and we should observe the effects of refund
        assert_eq!(Balances::free_balance(account(3)), 1000); // 1000 initial only because no quota available
        assert_eq!(Balances::free_balance(account(2)), 1500); // 1000 initial + 500 refunded
        assert_eq!(Balances::free_balance(account(1)), 1000); // only initial because no available weight to process
        assert_eq!(
            Balances::free_balance(<Test as pallet_quota::Config>::RefundAccount::get()),
            // initial minus refunds
            10_000 - 500
        );
        // events
        System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
            who: account(2),
            identity: 2,
            amount: 500,
        }));
    })
}
