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

#![cfg(not(feature = "constant-fees"))]

mod common;

use common::*;
use frame_support::{assert_ok, pallet_prelude::DispatchClass};
use gdev_runtime::*;
use sp_keyring::AccountKeyring;
use sp_runtime::Perquintill;

/// This test checks that an almost empty block incurs no fees for an extrinsic.
#[test]
fn test_fees_empty() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 10_000),
            (AccountKeyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: AccountKeyring::Eve.to_account_id().into(),
                value: 500,
            });

            let xt =
                common::get_unchecked_extrinsic(call, 4u64, 8u64, AccountKeyring::Alice, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));
            // The block is almost empty, so the extrinsic should incur no fee
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000 - 500
            );
        })
}

/// This test checks the fee behavior when the block is almost full.
/// - Multiple extrinsics are applied successfully without incurring fees until the block is under target weight.
/// - The last extrinsic incurs additional fees as the block reaches its target, verifying fee calculation under high load conditions.
#[test]
fn test_fees_weight() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 10_000),
            (AccountKeyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let mut transactions = 0u64;
            let weights = BlockWeights::get();
            let normal_max_weight = weights
                .get(DispatchClass::Normal)
                .max_total
                .unwrap_or(weights.max_block);
            // Stopping just below the limit
            while System::block_weight()
                .get(DispatchClass::Normal)
                .all_lt(Target::get() * normal_max_weight * Perbill::from_percent(99))
            {
                let call = RuntimeCall::System(SystemCall::remark {
                    remark: vec![255u8; 1],
                });
                let xt = get_unchecked_extrinsic(
                    call,
                    4u64,
                    8u64,
                    AccountKeyring::Alice,
                    0u64,
                    transactions as u32,
                );
                assert_ok!(Executive::apply_extrinsic(xt));
                transactions += 1;
            }
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000
            );
            // Ensure that the next extrinsic exceeds the limit.
            System::set_block_consumed_resources(Target::get() * normal_max_weight, 0_usize);
            // The block will reach the fee limit, so the next extrinsic should start incurring fees.
            let call = RuntimeCall::System(SystemCall::remark {
                remark: vec![255u8; 1],
            });

            let xt = get_unchecked_extrinsic(
                call,
                4u64,
                8u64,
                AccountKeyring::Alice,
                0u64,
                transactions as u32,
            );
            assert_ok!(Executive::apply_extrinsic(xt));
            assert_ne!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000
            );
        })
}

/// This test checks the fee behavior when the block is almost full.
/// - Multiple extrinsics are applied successfully without incurring fees until the block is under target length.
/// - The last extrinsic incurs additional fees as the block reaches its target, verifying fee calculation under high load conditions.
#[test]
fn test_fees_length() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 10_000),
            (AccountKeyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let mut transactions = 0u64;
            let length = BlockLength::get();
            let normal_max_length = *length.max.get(DispatchClass::Normal) as u64;

            // Stopping just below the limit
            while u64::from(System::all_extrinsics_len())
                < (Target::get() * Perquintill::from_percent(99) * normal_max_length)
            {
                let call = RuntimeCall::System(SystemCall::remark {
                    remark: vec![255u8; 1_000],
                });
                let xt = get_unchecked_extrinsic(
                    call,
                    4u64,
                    8u64,
                    AccountKeyring::Alice,
                    0u64,
                    transactions as u32,
                );
                assert_ok!(Executive::apply_extrinsic(xt));
                transactions += 1;
            }
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000
            );
            // Ensure that the next extrinsic exceeds the limit.
            System::set_block_consumed_resources(
                Weight::zero(),
                (Target::get() * normal_max_length).try_into().unwrap(),
            );
            // The block will reach the fee limit, so the next extrinsic should start incurring fees.
            let call = RuntimeCall::System(SystemCall::remark {
                remark: vec![255u8; 1],
            });

            let xt = get_unchecked_extrinsic(
                call,
                4u64,
                8u64,
                AccountKeyring::Alice,
                0u64,
                transactions as u32,
            );
            assert_ok!(Executive::apply_extrinsic(xt));
            assert_ne!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000
            );
        })
}

/// This test checks the behavior of the fee multiplier based on block weight
/// and previous block weight.
#[test]
fn test_fees_multiplier_weight() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 10_000),
            (AccountKeyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let weights = BlockWeights::get();
            let normal_max_weight = weights
                .get(DispatchClass::Normal)
                .max_total
                .unwrap_or(weights.max_block);

            assert_eq!(
                pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier(),
                1.into()
            );
            // If the block weight is over the target and the previous block was also over the target,
            // the fee multiplier is increased by one, up to the MaxMultiplier.
            let mut current = 0u128;
            for i in 1..20u128 {
                System::set_block_consumed_resources(Target::get() * normal_max_weight, 0_usize);
                run_to_block(i as u32);
                current += 1;
                assert_eq!(
                    pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier(),
                    core::cmp::min(current.into(), MaxMultiplier::get())
                );
            }

            // If the block weight is under the target and the previous block was also under the target,
            // the fee multiplier is decreased by one, down to the one.
            let mut current = 10u128;
            for i in 20..50u32 {
                run_to_block(i);
                current = current.saturating_sub(1).max(1u128);
                assert_eq!(
                    pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier(),
                    current.into()
                );
            }
        })
}

/// This test checks the behavior of the fee multiplier based on block length
/// and previous block length.
#[test]
fn test_fees_multiplier_length() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 10_000),
            (AccountKeyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let length = BlockLength::get();
            let normal_max_length = *length.max.get(DispatchClass::Normal) as u64;

            assert_eq!(
                pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier(),
                1.into()
            );
            // If the block weight is over the target and the previous block was also over the target,
            // the fee multiplier is increased by one, up to the MaxMultiplier.
            let mut current = 0u128;
            for i in 1..20u128 {
                System::set_block_consumed_resources(
                    Weight::zero(),
                    (Target::get() * normal_max_length).try_into().unwrap(),
                );
                run_to_block(i as u32);
                current += 1;
                assert_eq!(
                    pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier(),
                    core::cmp::min(current.into(), MaxMultiplier::get())
                );
            }

            // If the block weight is under the target and the previous block was also under the target,
            // the fee multiplier is decreased by one, down to the one.
            let mut current = 10u128;
            for i in 20..50u32 {
                run_to_block(i);
                current = current.saturating_sub(1).max(1u128);
                assert_eq!(
                    pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier(),
                    current.into()
                );
            }
        })
}
