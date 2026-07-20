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

#![cfg(feature = "constant-fees")]

mod common;

use common::*;
use frame_support::{
    assert_ok,
    traits::{OnIdle, StoredMap},
};
use gdev_runtime::*;
use sp_core::Encode;
use sp_keyring::sr25519::Keyring;
use sp_runtime::{
    MultiAddress,
    transaction_validity::{InvalidTransaction, TransactionValidityError},
};

/// test currency transfer with extrinsic
// the signer account should pay fees and a tip
// the treasury should get the fees
#[test]
fn test_transfer_xt() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 500,
            });

            // 1 cĞD of tip
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 1u64, 0);
            // let info = xt.get_dispatch_info();
            // println!("dispatch info:\n\t {:?}\n", info);

            assert_eq!(Balances::free_balance(Treasury::account_id()), 100);
            // Alice gives 500 to Eve
            assert_ok!(Executive::apply_extrinsic(xt));
            // check amounts
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                10_000 - 500 - 3 // initial - transfered - fees
            );
            assert_eq!(
                Balances::free_balance(Keyring::Eve.to_account_id()),
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
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 500,
            });

            // 1 cĞD of tip
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 1u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            // check that refund was added to the queue
            assert_eq!(
                pallet_quota::RefundQueue::<Runtime>::get()
                    .first()
                    .expect("a refund should have been added to the queue"),
                &pallet_quota::pallet::Refund {
                    account: Keyring::Alice.to_account_id(),
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
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 500,
            });

            // 1 cĞD of tip
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 1u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            let refund = pallet_quota::RefundQueue::<Runtime>::get()
                .first()
                .cloned()
                .expect("a refund should have been added to the queue");
            let expected_refund = Quota::estimate_quota_refund(1u32).min(refund.amount);

            // call on_idle to activate refund
            Quota::on_idle(System::block_number(), Weight::from(1_000_000_000));

            // check that refund event existed
            System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
                who: Keyring::Alice.to_account_id(),
                identity: 1u32,
                amount: expected_refund,
            }));

            // check that refund queue is empty
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                10_000 - 500 - 1 - 2 + expected_refund // initial - transfered - tip - fees + refunded fees
            );
        })
}

/// test no refund when no identity linked
#[test]
fn test_no_refund() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // Eve → Alice
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Alice.to_account_id().into(),
                value: 500,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Eve, 1u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));
            // check that refund queue is empty
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
            assert_eq!(Balances::free_balance(Treasury::account_id()), 100 + 3);
        })
}

/// test refund on_idle when linked account is reaped
#[test]
fn test_refund_reaped_linked_account() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Ferdie.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let genesis_hash = System::block_hash(0);
            let alice = Keyring::Alice.to_account_id();
            let ferdie = Keyring::Ferdie.to_account_id();
            let payload = (b"link", genesis_hash, 1u32, ferdie.clone()).encode();
            let signature = Keyring::Ferdie.sign(&payload);

            // Ferdie's account can be linked to Alice identity
            assert_ok!(Identity::link_account(
                RuntimeOrigin::signed(alice.clone()),
                ferdie.clone(),
                signature.into()
            ));
            assert_eq!(
                frame_system::Pallet::<Runtime>::get(&ferdie).linked_idty,
                Some(1)
            );

            // transfer_all call to extrinsic
            let call = RuntimeCall::Balances(BalancesCall::transfer_all {
                dest: Keyring::Alice.to_account_id().into(),
                keep_alive: false,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Ferdie, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            assert_eq!(Balances::free_balance(ferdie.clone()), 0);
            // During reaping the account is unlinked
            assert!(
                frame_system::Pallet::<Runtime>::get(&ferdie)
                    .linked_idty
                    .is_none()
            );

            // since the account is reaped, it is not linked anymore and no refund is added to queue
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
        })
}

/// test no refund on_idle when account is not a member
#[test]
fn test_no_member_no_refund() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Bob.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // Revoked identities are not eligible for a refund
            let revocation_payload = pallet_identity::RevocationPayload {
                idty_index: 2u32,
                genesis_hash: System::block_hash(0),
            };
            let signature = Keyring::Bob.sign(
                &(
                    pallet_identity::REVOCATION_PAYLOAD_PREFIX,
                    revocation_payload,
                )
                    .encode(),
            );
            assert_ok!(Identity::revoke_identity(
                RuntimeOrigin::signed(Keyring::Bob.to_account_id()),
                2,
                Keyring::Bob.to_account_id(),
                signature.into()
            ));
            assert_eq!(
                pallet_identity::Identities::<Runtime>::get(&2)
                    .unwrap()
                    .status,
                pallet_identity::IdtyStatus::Revoked
            );
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Ferdie.to_account_id().into(),
                value: 500,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Bob, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());

            // Unconfirmed identities are not eligible for a refund
            assert_ok!(Identity::create_identity(
                RuntimeOrigin::signed(Keyring::Alice.to_account_id()),
                Keyring::Ferdie.to_account_id(),
            ));
            assert_eq!(
                pallet_identity::Identities::<Runtime>::get(&5)
                    .unwrap()
                    .status,
                pallet_identity::IdtyStatus::Unconfirmed
            );
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Alice.to_account_id().into(),
                value: 500,
            });
            let xt = get_unchecked_extrinsic(call.clone(), 4u64, 8u64, Keyring::Ferdie, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());

            // Unvalidated identities are not eligible for a refund
            assert_ok!(Identity::confirm_identity(
                RuntimeOrigin::signed(Keyring::Ferdie.to_account_id()),
                "ferdie".into(),
            ));
            assert_eq!(
                pallet_identity::Identities::<Runtime>::get(&5)
                    .unwrap()
                    .status,
                pallet_identity::IdtyStatus::Unvalidated
            );
            let xt = get_unchecked_extrinsic(call.clone(), 4u64, 8u64, Keyring::Ferdie, 0u64, 1);
            assert_ok!(Executive::apply_extrinsic(xt));
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());

            // NotMember identities are not eligible for a refund
            pallet_identity::Pallet::<Runtime>::membership_removed(1);
            assert_eq!(
                pallet_identity::Identities::<Runtime>::get(&1)
                    .unwrap()
                    .status,
                pallet_identity::IdtyStatus::NotMember
            );
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Bob.to_account_id().into(),
                value: 500,
            });
            let xt = get_unchecked_extrinsic(call.clone(), 4u64, 8u64, Keyring::Alice, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
        })
}

/// test that consume_oneshot_account via extrinsic withdraws fees from oneshot storage
// when account has both regular balance and oneshot balance, fees come from oneshot storage
#[test]
fn test_oneshot_consume_fee_from_oneshot_storage() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // Alice creates a oneshot account for Eve with 500
            assert_ok!(OneshotAccount::create_oneshot_account(
                RuntimeOrigin::signed(Keyring::Alice.to_account_id()),
                MultiAddress::Id(Keyring::Eve.to_account_id()),
                500
            ));
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                10_000 - 500
            );
            assert_eq!(
                pallet_oneshot_account::OneshotAccounts::<Runtime>::get(
                    Keyring::Eve.to_account_id()
                ),
                Some(500)
            );

            // Eve consumes her oneshot account, sending to Alice (normal account)
            let call = RuntimeCall::OneshotAccount(
                pallet_oneshot_account::Call::consume_oneshot_account {
                    block_height: 0u32.into(),
                    dest: pallet_oneshot_account::Account::Normal(MultiAddress::Id(
                        Keyring::Alice.to_account_id(),
                    )),
                },
            );
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Eve, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            // Oneshot account consumed
            assert!(
                pallet_oneshot_account::OneshotAccounts::<Runtime>::get(
                    Keyring::Eve.to_account_id()
                )
                .is_none()
            );
            // Eve's regular balance is unchanged (fees came from oneshot storage)
            assert_eq!(Balances::free_balance(Keyring::Eve.to_account_id()), 10_000);
            // Alice receives the oneshot value minus the fee deducted from oneshot storage
            // Under constant-fees: fee = 2, so Alice gets 500 - 2 = 498
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                9_500 + 498
            );
            // Eve has no linked identity, so no refund is queued
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
        })
}

/// test that consume_oneshot_account by an identity-linked account queues a refund
// fees are withdrawn from oneshot storage, but refund is queued via the duniter-account
// correct_and_deposit_fee delegation since the account has a linked identity
#[test]
fn test_oneshot_consume_with_linked_identity_gets_refund() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Bob.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // Alice (identity 1, Member) creates oneshot account for herself
            assert_ok!(OneshotAccount::create_oneshot_account(
                RuntimeOrigin::signed(Keyring::Alice.to_account_id()),
                MultiAddress::Id(Keyring::Alice.to_account_id()),
                500
            ));
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                10_000 - 500
            );

            // Alice consumes her own oneshot account, sending to Bob
            let call = RuntimeCall::OneshotAccount(
                pallet_oneshot_account::Call::consume_oneshot_account {
                    block_height: 0u32.into(),
                    dest: pallet_oneshot_account::Account::Normal(MultiAddress::Id(
                        Keyring::Bob.to_account_id(),
                    )),
                },
            );
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            // Oneshot consumed
            assert!(
                pallet_oneshot_account::OneshotAccounts::<Runtime>::get(
                    Keyring::Alice.to_account_id()
                )
                .is_none()
            );
            // Bob receives oneshot value minus fee (500 - 2 = 498)
            assert_eq!(
                Balances::free_balance(Keyring::Bob.to_account_id()),
                10_000 + 498
            );
            // Alice has linked identity 1 → refund should be queued
            // refund amount = corrected_fee - tip = 2 - 0 = 2
            let queue = pallet_quota::RefundQueue::<Runtime>::get();
            assert_eq!(queue.len(), 1);
            assert_eq!(
                queue.first().unwrap(),
                &pallet_quota::pallet::Refund {
                    account: Keyring::Alice.to_account_id(),
                    identity: 1u32,
                    amount: 2u64
                }
            );
        })
}

/// test that tips are excluded from refund amount
// two extrinsics from the same identity-linked account: one without tip, one with tip
// both should produce the same refund amount (base fee only)
#[test]
fn test_tip_excluded_from_refund_amount() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // First transfer with no tip
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 100,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            // refund amount = corrected_fee(2) - tip(0) = 2
            let queue = pallet_quota::RefundQueue::<Runtime>::get();
            assert_eq!(queue.len(), 1);
            assert_eq!(queue[0].amount, 2u64);

            // Second transfer with tip=2
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 100,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 2u64, 1);
            assert_ok!(Executive::apply_extrinsic(xt));

            // refund amount = corrected_fee(4) - tip(2) = 2 (same as without tip)
            let queue = pallet_quota::RefundQueue::<Runtime>::get();
            assert_eq!(queue.len(), 2);
            assert_eq!(queue[0].amount, 2u64); // first: no tip
            assert_eq!(queue[1].amount, 2u64); // second: with tip, but refund excludes tip

            // Alice paid: 100 + 2 (first) + 100 + 4 (second) = 206
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                10_000 - 206
            );
            // Treasury received all fees+tips: 2 + 4 = 6
            assert_eq!(Balances::free_balance(Treasury::account_id()), 100 + 6);
        })
}

/// test refund queue accumulation from multiple extrinsics in the same block
// multiple identity-linked accounts submit extrinsics, refunds accumulate and are processed
#[test]
fn test_multiple_extrinsics_refund_accumulation() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Bob.to_account_id(), 10_000),
            (Keyring::Eve.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            // Alice (identity 1) sends transfer
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 100,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            // Bob (identity 2) sends transfer
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 200,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Bob, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));

            // Alice sends another transfer
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Eve.to_account_id().into(),
                value: 100,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Alice, 0u64, 1);
            assert_ok!(Executive::apply_extrinsic(xt));

            // Refund queue should have 3 entries
            let queue = pallet_quota::RefundQueue::<Runtime>::get();
            assert_eq!(queue.len(), 3);
            assert_eq!(
                queue[0],
                pallet_quota::pallet::Refund {
                    account: Keyring::Alice.to_account_id(),
                    identity: 1u32,
                    amount: 2u64
                }
            );
            assert_eq!(
                queue[1],
                pallet_quota::pallet::Refund {
                    account: Keyring::Bob.to_account_id(),
                    identity: 2u32,
                    amount: 2u64
                }
            );
            assert_eq!(
                queue[2],
                pallet_quota::pallet::Refund {
                    account: Keyring::Alice.to_account_id(),
                    identity: 1u32,
                    amount: 2u64
                }
            );

            // Process all refunds
            Quota::on_idle(System::block_number(), Weight::from(1_000_000_000));

            // Queue should be empty after processing
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());

            // Alice: 10000 - 100 - 2 - 100 - 2 + refunds
            // Bob: 10000 - 200 - 2 + refund
            // Eve: 10000 + 100 + 200 + 100 (receives all transfers)
            assert_eq!(
                Balances::free_balance(Keyring::Eve.to_account_id()),
                10_000 + 400
            );
        })
}

/// test that can_withdraw_fee rejects extrinsic when balance is insufficient
// Eve has exactly the existential deposit (100) but cannot pay fees (2) without
// going below ED, so the fee withdrawal should be rejected
#[test]
fn test_insufficient_balance_rejects_extrinsic() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            // Eve gets exactly ED (100), not enough to cover fee (2) while staying alive
            (Keyring::Eve.to_account_id(), 100),
        ])
        .build()
        .execute_with(|| {
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Alice.to_account_id().into(),
                value: 0,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Eve, 0u64, 0);

            // Extrinsic should fail due to insufficient balance for fees
            let result = Executive::apply_extrinsic(xt);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                TransactionValidityError::Invalid(InvalidTransaction::Payment)
            );

            // Eve's balance should be unchanged
            assert_eq!(Balances::free_balance(Keyring::Eve.to_account_id()), 100);
            // No refund queued
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
        })
}

/// test that dynamically linking an identity enables fee refunds for subsequent transactions
// first transfer has no refund (no identity), then link identity, second transfer gets refund
#[test]
fn test_dynamic_link_identity_then_refund() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (Keyring::Alice.to_account_id(), 10_000),
            (Keyring::Ferdie.to_account_id(), 10_000),
        ])
        .build()
        .execute_with(|| {
            let ferdie = Keyring::Ferdie.to_account_id();

            // Ferdie has no linked identity
            assert!(
                frame_system::Pallet::<Runtime>::get(&ferdie)
                    .linked_idty
                    .is_none()
            );

            // First transfer: Ferdie → Alice, no identity linked → no refund
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Alice.to_account_id().into(),
                value: 100,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Ferdie, 0u64, 0);
            assert_ok!(Executive::apply_extrinsic(xt));
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());

            // Link Ferdie's account to Alice's identity (identity 1)
            let genesis_hash = System::block_hash(0);
            let payload = (b"link", genesis_hash, 1u32, ferdie.clone()).encode();
            let signature = Keyring::Ferdie.sign(&payload);
            assert_ok!(Identity::link_account(
                RuntimeOrigin::signed(Keyring::Alice.to_account_id()),
                ferdie.clone(),
                signature.into()
            ));
            assert_eq!(
                frame_system::Pallet::<Runtime>::get(&ferdie).linked_idty,
                Some(1)
            );

            // Second transfer: Ferdie → Alice, now with linked identity → refund queued
            let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: Keyring::Alice.to_account_id().into(),
                value: 100,
            });
            let xt = get_unchecked_extrinsic(call, 4u64, 8u64, Keyring::Ferdie, 0u64, 1);
            assert_ok!(Executive::apply_extrinsic(xt));

            // Now refund should be queued for Ferdie with Alice's identity
            let queue = pallet_quota::RefundQueue::<Runtime>::get();
            assert_eq!(queue.len(), 1);
            assert_eq!(
                queue[0],
                pallet_quota::pallet::Refund {
                    account: ferdie,
                    identity: 1u32,
                    amount: 2u64
                }
            );
        })
}
