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

            // call on_idle to activate refund
            Quota::on_idle(System::block_number(), Weight::from(1_000_000_000));

            // check that refund event existed
            System::assert_has_event(RuntimeEvent::Quota(pallet_quota::Event::Refunded {
                who: Keyring::Alice.to_account_id(),
                identity: 1u32,
                amount: 1u64,
            }));

            // check that refund queue is empty
            assert!(pallet_quota::RefundQueue::<Runtime>::get().is_empty());
            assert_eq!(
                Balances::free_balance(Keyring::Alice.to_account_id()),
                10_000 - 500 - 1 - 2 + 1 // initial - transfered - tip - fees + refunded fees
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
            assert!(frame_system::Pallet::<Runtime>::get(&ferdie)
                .linked_idty
                .is_none());

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
