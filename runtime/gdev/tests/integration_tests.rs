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

mod common;

use common::*;
use frame_support::traits::{Get, PalletInfo, StorageInfo, StorageInfoTrait};
use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageHasher, Twox128};
use gdev_runtime::*;
use sp_keyring::AccountKeyring;
use sp_runtime::MultiAddress;

#[test]
fn verify_treasury_account() {
    // println!("{}", Treasury::account_id());
}

#[test]
fn verify_pallet_prefixes() {
    let prefix = |pallet_name, storage_name| {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
        res[16..32].copy_from_slice(&Twox128::hash(storage_name));
        res.to_vec()
    };
    assert_eq!(
        <Timestamp as StorageInfoTrait>::storage_info(),
        vec![
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"Now".to_vec(),
                prefix: prefix(b"Timestamp", b"Now"),
                max_values: Some(1),
                max_size: Some(8),
            },
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"DidUpdate".to_vec(),
                prefix: prefix(b"Timestamp", b"DidUpdate"),
                max_values: Some(1),
                max_size: Some(1),
            }
        ]
    );
}

#[test]
fn verify_pallet_indices() {
    fn is_pallet_index<P: 'static>(index: usize) {
        assert_eq!(
            <Runtime as frame_system::Config>::PalletInfo::index::<P>(),
            Some(index)
        );
    }
    is_pallet_index::<System>(0);
}

#[test]
fn verify_proxy_type_indices() {
    assert_eq!(ProxyType::AlmostAny as u8, 0);
}

#[test]
fn test_genesis_build() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);
    });
}

/// test calling remove_identity
#[test]
fn test_remove_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            4,
            None
        ));

        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRevoked(4),
        ));
        System::assert_has_event(RuntimeEvent::System(frame_system::Event::KilledAccount {
            account: AccountKeyring::Dave.to_account_id(),
        }));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved { idty_index: 4 },
        ));
    });
}

/// test identity is validated when membership is claimed
#[test]
fn test_validate_identity_when_claim() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Ferdie.to_account_id(), 1000)])
        .build()
        .execute_with(|| {
            run_to_block(1);
            // alice create identity for Eve
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));
            run_to_block(2);
            // eve confirms her identity
            assert_ok!(Identity::confirm_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                "Eeeeeveeeee".into(),
            ));
            run_to_block(3);
            // eve gets certified by bob and charlie
            assert_ok!(Cert::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                2,
                5
            ));
            assert_ok!(Cert::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into(),
                3,
                5
            ));

            // eve can claim her membership
            assert_ok!(Membership::claim_membership(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            ));

            System::assert_has_event(RuntimeEvent::Membership(
                pallet_membership::Event::MembershipAcquired(5),
            ));

            // ferdie can not validate eve identity because already validated
            assert_noop!(
                Identity::validate_identity(
                    frame_system::RawOrigin::Signed(AccountKeyring::Ferdie.to_account_id()).into(),
                    5,
                ),
                pallet_identity::Error::<Runtime>::IdtyAlreadyValidated
            );
        });
}

/// test membership expiry
#[test]
fn test_membership_expiry() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(100);
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipExpired(1),
        ));
        // membership expiry should not trigger identity removal
        assert!(!System::events().iter().any(|record| record.event
            == RuntimeEvent::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 1 })));
    });
}

/// test membership renewal
#[test]
fn test_membership_renewal() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        // renew at block 2
        run_to_block(2);
        assert_ok!(Membership::renew_membership(
            frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
        ));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRenewed(1),
        ));

        // renew at block 3
        run_to_block(3);
        assert_ok!(Membership::renew_membership(
            frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
        ));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRenewed(1),
        ));

        // should expire at block 103 = 3+100
        run_to_block(103);
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipExpired(1),
        ));
    });
}

// test that UD are auto claimed when identity is removed
#[test]
fn test_remove_identity_after_one_ud() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        //println!("UdCreationPeriod={}", <Runtime as pallet_universal_dividend::Config>::UdCreationPeriod::get());
        run_to_block(<Runtime as pallet_universal_dividend::Config>::UdCreationPeriod::get() + 1);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            4,
            None
        ));

        // Verify events
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRevoked(4),
        ));
        System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Deposit {
            who: AccountKeyring::Dave.to_account_id(),
            amount: 1_000,
        }));
        System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Endowed {
            account: AccountKeyring::Dave.to_account_id(),
            free_balance: 1_000,
        }));
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::UdsAutoPaidAtRemoval {
                count: 1,
                total: 1_000,
                who: AccountKeyring::Dave.to_account_id(),
            },
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved { idty_index: 4 },
        ));

        // Verify state
        assert!(Identity::identity(4).is_none());
        assert_eq!(
            Balances::free_balance(AccountKeyring::Dave.to_account_id()),
            1_000
        );
    });
}

/// test that UD are auto claimed when membership expires
/// and that claimed UD matches expectations
#[test]
fn test_ud_claimed_membership_on_and_off() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        // UD are created every 10 blocks from block 0
        run_to_block(10);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::NewUdCreated {
                amount: 1000,
                index: 1,
                monetary_mass: 4000,
                members_count: 4,
            },
        ));
        // UD not claimed, still initial balance to 0
        assert_eq!(
            Balances::free_balance(AccountKeyring::Alice.to_account_id()),
            0
        );

        run_to_block(11);
        // alice identity expires
        assert_ok!(Membership::force_expire_membership(1));
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::UdsAutoPaidAtRemoval {
                count: 1,
                total: 1_000,
                who: AccountKeyring::Alice.to_account_id(),
            },
        ));
        // alice balances should be increased by 1 UD
        assert_eq!(
            Balances::free_balance(AccountKeyring::Alice.to_account_id()),
            1000
        );

        // UD number 2
        run_to_block(20);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::NewUdCreated {
                amount: 1000,
                index: 2,
                monetary_mass: 7000, // 4000 + 3 × 1000
                members_count: 3,    // alice is not member at this UD
            },
        ));

        // alice claims back her membership
        assert_ok!(Membership::claim_membership(
            frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into()
        ));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipAcquired(1),
        ));

        // UD number 3
        run_to_block(30);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::NewUdCreated {
                amount: 1000,
                index: 3,
                monetary_mass: 11000, // 7000 + 4 × 1000
                members_count: 4,     // alice is member again at this UD
            },
        ));

        // one block later, alice claims her new UD
        run_to_block(31);
        assert_ok!(UniversalDividend::claim_uds(
            frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into()
        ));
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::UdsClaimed {
                count: 1,
                total: 1_000,
                who: AccountKeyring::Alice.to_account_id(),
            },
        ));
        assert_eq!(
            Balances::free_balance(AccountKeyring::Alice.to_account_id()),
            2000 // one more UD
        );

        // println!("{:?}", System::events());
    });
}

/// test when root removes and identity, all consumers should be deleted
#[test]
fn test_remove_smith_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            3,
            None
        ));
        // Verify events
        System::assert_has_event(RuntimeEvent::SmithMembership(
            pallet_membership::Event::MembershipRevoked(3),
        ));
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberRemoved(3),
        ));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRevoked(3),
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved { idty_index: 3 },
        ));
    });
}

#[test]
fn test_create_new_account_with_insufficient_balance() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(2);

            // Should be able to transfer 4 units to a new account
            assert_ok!(Balances::transfer(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                400
            ));

            System::assert_has_event(RuntimeEvent::System(frame_system::Event::NewAccount {
                account: AccountKeyring::Eve.to_account_id(),
            }));
            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Endowed {
                account: AccountKeyring::Eve.to_account_id(),
                free_balance: 400,
            }));
            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Transfer {
                from: AccountKeyring::Alice.to_account_id(),
                to: AccountKeyring::Eve.to_account_id(),
                amount: 400,
            }));

            // At next block, the new account must be reaped because its balance is not sufficient
            // to pay the "new account tax"
            run_to_block(3);

            System::assert_has_event(RuntimeEvent::Account(
                pallet_duniter_account::Event::ForceDestroy {
                    who: AccountKeyring::Eve.to_account_id(),
                    balance: 400,
                },
            ));
            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Deposit {
                who: Treasury::account_id(),
                amount: 400,
            }));
            System::assert_has_event(RuntimeEvent::Treasury(pallet_treasury::Event::Deposit {
                value: 400,
            }));

            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                0
            );
            assert_eq!(Balances::free_balance(Treasury::account_id()), 600);
        });
}

#[test]
fn test_create_new_account() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(2);

            // Should be able to transfer 5 units to a new account
            assert_ok!(Balances::transfer(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                500
            ));
            //println!("{:#?}", System::events());
            System::assert_has_event(RuntimeEvent::System(frame_system::Event::NewAccount {
                account: AccountKeyring::Eve.to_account_id(),
            }));
            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Endowed {
                account: AccountKeyring::Eve.to_account_id(),
                free_balance: 500,
            }));
            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Transfer {
                from: AccountKeyring::Alice.to_account_id(),
                to: AccountKeyring::Eve.to_account_id(),
                amount: 500,
            }));

            // At next block, the new account must be created,
            // and new account tax should be collected and deposited in the treasury
            run_to_block(3);

            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Withdraw {
                who: AccountKeyring::Eve.to_account_id(),
                amount: 300,
            }));
            System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Deposit {
                who: Treasury::account_id(),
                amount: 300,
            }));
            System::assert_has_event(RuntimeEvent::Treasury(pallet_treasury::Event::Deposit {
                value: 300,
            }));

            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                200
            );
            assert_eq!(Balances::free_balance(Treasury::account_id()), 500);

            // A random id request should be registered
            assert_eq!(
                Account::pending_random_id_assignments(0),
                Some(AccountKeyring::Eve.to_account_id())
            );

            // We can't remove the account until the random id is assigned
            run_to_block(4);
            assert_noop!(
                Balances::transfer(
                    frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                    MultiAddress::Id(AccountKeyring::Alice.to_account_id()),
                    200
                ),
                pallet_balances::Error::<Runtime>::KeepAlive,
            );
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                200
            );
            assert_ok!(Balances::transfer_all(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Alice.to_account_id()),
                false
            ),);
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                200
            );
        });
}

#[test]
fn test_create_new_idty() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(2);

            // Should be able to create an identity
            assert_ok!(Balances::transfer(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                200
            ));
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));

            // At next block, nothing should be preleved
            run_to_block(3);
            let events = System::events();
            assert_eq!(events.len(), 0);

            // A random id request should be registered
            assert_eq!(
                Account::pending_random_id_assignments(0),
                Some(AccountKeyring::Eve.to_account_id())
            );
        });
}

#[test]
fn test_create_new_idty_without_founds() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(2);

            // Should be able to create an identity without founds
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));

            // At next block, nothing should be preleved
            run_to_block(3);
            let events = System::events();
            assert_eq!(events.len(), 0);

            // Deposit some founds on the identity account,
            // this should trigger the random id assignemt
            assert_ok!(Balances::transfer(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                200
            ));

            // At next block, nothing should be preleved,
            // and a random id request should be registered
            run_to_block(4);
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                200
            );
            assert_eq!(
                Account::pending_random_id_assignments(0),
                Some(AccountKeyring::Eve.to_account_id())
            );
        });
}

/// test that newly validated identity gets initialized with the next UD
#[test]
fn test_validate_new_idty_after_few_uds() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 1_000),
            (AccountKeyring::Bob.to_account_id(), 1_000),
            (AccountKeyring::Charlie.to_account_id(), 1_000),
            (AccountKeyring::Eve.to_account_id(), 1_000),
        ])
        .build()
        .execute_with(|| {
            run_to_block(21);

            // Should be able to create an identity
            assert_ok!(Balances::transfer(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                200
            ));
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));

            // At next block, the created identity should be confirmed by its owner
            run_to_block(22);
            assert_ok!(Identity::confirm_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                pallet_identity::IdtyName::from("Eve"),
            ));

            // At next block, Bob should be able to certify and validate the new identity
            run_to_block(23);
            assert_ok!(Cert::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                2,
                5,
            ));
            assert_ok!(Identity::validate_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                5,
            ));

            // The new member should have first_eligible_ud equal to three
            assert!(Identity::identity(5).is_some());
            assert_eq!(
                Identity::identity(5).unwrap().data,
                IdtyData {
                    // first eligible UD will be at block 30
                    first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::from(3),
                }
            );
        });
}

/// test that newly validated identity gets initialized with the next UD
// even when the method used is membership claim
#[test]
fn test_claim_memberhsip_after_few_uds() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 1_000),
            (AccountKeyring::Bob.to_account_id(), 1_000),
            (AccountKeyring::Charlie.to_account_id(), 1_000),
            (AccountKeyring::Eve.to_account_id(), 1_000),
        ])
        .build()
        .execute_with(|| {
            run_to_block(21);

            // Should be able to create an identity
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));

            // At next block, the created identity should be confirmed by its owner
            run_to_block(22);
            assert_ok!(Identity::confirm_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                pallet_identity::IdtyName::from("Eve"),
            ));

            // At next block, Bob should be able to certify the new identity
            run_to_block(23);
            assert_ok!(Cert::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                2,
                5,
            ));

            // eve should be able to claim her membership
            assert_ok!(Membership::claim_membership(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            ));

            // The new member should have first_eligible_ud equal to three
            assert!(Identity::identity(5).is_some());
            assert_eq!(
                Identity::identity(5).unwrap().data,
                IdtyData {
                    // first eligible UD will be at block 30
                    first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::from(3),
                }
            );
        });
}

/// test oneshot accounts
#[test]
fn test_oneshot_accounts() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 1_000),
            (AccountKeyring::Eve.to_account_id(), 1_000),
        ])
        .build()
        .execute_with(|| {
            run_to_block(6);

            assert_ok!(OneshotAccount::create_oneshot_account(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                400
            ));
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                600
            );
            run_to_block(7);

            assert_ok!(OneshotAccount::consume_oneshot_account_with_remaining(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                0,
                pallet_oneshot_account::Account::Oneshot(MultiAddress::Id(
                    AccountKeyring::Ferdie.to_account_id()
                )),
                pallet_oneshot_account::Account::Normal(MultiAddress::Id(
                    AccountKeyring::Alice.to_account_id()
                )),
                300
            ));
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                700
            );
            assert_noop!(
                OneshotAccount::consume_oneshot_account(
                    frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                    0,
                    pallet_oneshot_account::Account::Oneshot(MultiAddress::Id(
                        AccountKeyring::Ferdie.to_account_id()
                    )),
                ),
                pallet_oneshot_account::Error::<Runtime>::OneshotAccountNotExist
            );
            run_to_block(8);
            // Oneshot account consumption should not increment the nonce
            assert_eq!(
                System::account(AccountKeyring::Eve.to_account_id()).nonce,
                0
            );

            assert_ok!(OneshotAccount::consume_oneshot_account(
                frame_system::RawOrigin::Signed(AccountKeyring::Ferdie.to_account_id()).into(),
                0,
                pallet_oneshot_account::Account::Normal(MultiAddress::Id(
                    AccountKeyring::Alice.to_account_id()
                )),
            ));
            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                1000
            );
            assert_noop!(
                OneshotAccount::consume_oneshot_account(
                    frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                    0,
                    pallet_oneshot_account::Account::Normal(MultiAddress::Id(
                        AccountKeyring::Alice.to_account_id()
                    )),
                ),
                pallet_oneshot_account::Error::<Runtime>::OneshotAccountNotExist
            );
        });
}
