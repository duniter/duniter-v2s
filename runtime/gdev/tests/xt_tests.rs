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

// these integration tests aim to test fees and extrinsic-related externalities
// they need constant-fees feature to work

mod common;

use common::*;
use frame_support::assert_ok;
use frame_support::traits::OnIdle;
use gdev_runtime::*;
use sp_core::Encode;
use sp_core::Pair;
use sp_keyring::AccountKeyring;
use sp_runtime::generic::SignedPayload;
use sp_runtime::traits::Extrinsic;

/// get extrinsic for given call
fn get_unchecked_extrinsic(
    call: RuntimeCall,
    era: u64,
    block: u64,
    signer: AccountKeyring,
    tip: Balance,
) -> UncheckedExtrinsic {
    let extra: gdev_runtime::SignedExtra = (
        frame_system::CheckNonZeroSender::<gdev_runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<gdev_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<gdev_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<gdev_runtime::Runtime>::new(),
        frame_system::CheckMortality::<gdev_runtime::Runtime>::from(
            sp_runtime::generic::Era::mortal(era, block),
        ),
        frame_system::CheckNonce::<gdev_runtime::Runtime>::from(0u32).into(),
        frame_system::CheckWeight::<gdev_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<gdev_runtime::Runtime>::from(tip),
    );
    let payload = SignedPayload::new(call.clone(), extra.clone()).unwrap();
    let origin = signer;
    let sig = payload.using_encoded(|payload| origin.pair().sign(payload));

    UncheckedExtrinsic::new(
        call,
        Some((origin.to_account_id().into(), sig.into(), extra)),
    )
    .unwrap()
}

/// test currency transfer with extrinsic
// the signer account should pay fees and a tip
// the treasury should get the fees
#[test]
fn test_transfer_xt() {
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

            // 1 cĞD of tip
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, AccountKeyring::Alice, 1u64);
            // let info = xt.get_dispatch_info();
            // println!("dispatch info:\n\t {:?}\n", info);

            assert_eq!(Balances::free_balance(Treasury::account_id()), 100);
            // Alice gives 500 to Eve
            assert_ok!(Executive::apply_extrinsic(xt));
            // check amounts
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000 - 500 - 3 // initial - transfered - fees
            );
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                10_000 + 500 // initial + transfered
            );
            assert_eq!(Balances::free_balance(Treasury::account_id()), 100 + 3);
        })
}

/// test that fees are added to the refund queue
#[test]
fn test_refund_queue() {
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

            // 1 cĞD of tip
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, AccountKeyring::Alice, 1u64);
            assert_ok!(Executive::apply_extrinsic(xt));

            // check that refund was added to the queue
            assert_eq!(
                pallet_quota::RefundQueue::<Runtime>::get()
                    .first()
                    .expect("a refund should have been added to the queue"),
                &pallet_quota::pallet::Refund {
                    account: AccountKeyring::Alice.to_account_id(),
                    identity: 1u32,
                    amount: 2u64
                }
            );
        })
}

/// test refund on_idle
#[test]
fn test_refund_on_idle() {
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

            // 1 cĞD of tip
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, AccountKeyring::Alice, 1u64);
            assert_ok!(Executive::apply_extrinsic(xt));

            // call on_idle to activate refund
            Quota::on_idle(System::block_number(), Weight::from(1_000_000_000));

            // check that refund event existed
            System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
                who: AccountKeyring::Alice.to_account_id(),
                identity: 1u32,
                amount: 1u64,
            }));

            // check that refund queue is empty
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                10_000 - 500 - 1 - 2 + 1 // initial - transfered - tip - fees + refunded fees
            );
        })
}

/// test no refund when no identity linked
#[test]
fn test_no_refund() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 10_000),
            (AccountKeyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // Eve → Alice
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: AccountKeyring::Alice.to_account_id().into(),
                value: 500,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, AccountKeyring::Eve, 1u64);
            assert_ok!(Executive::apply_extrinsic(xt));
            // check that refund queue is empty
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
            assert_eq!(Balances::free_balance(Treasury::account_id()), 100 + 3);
        })
}
