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
    println!("{}", Treasury::account_id());
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

#[test]
fn test_remove_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            4,
            None
        ));
        let events = System::events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            System::events()[0].event,
            Event::Membership(pallet_membership::Event::MembershipRevoked(4))
        );
        assert_eq!(
            System::events()[1].event,
            Event::System(frame_system::Event::KilledAccount {
                account: AccountKeyring::Dave.to_account_id()
            })
        );
        assert_eq!(
            System::events()[2].event,
            Event::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 4 })
        );
    });
}
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
        let events = System::events();
        //println!("{:?}", events);
        assert_eq!(events.len(), 5);
        assert_eq!(
            System::events()[0].event,
            Event::Membership(pallet_membership::Event::MembershipRevoked(4))
        );
        assert_eq!(
            System::events()[1].event,
            Event::Balances(pallet_balances::Event::Deposit {
                who: AccountKeyring::Dave.to_account_id(),
                amount: 1_000
            })
        );
        assert_eq!(
            System::events()[2].event,
            Event::Balances(pallet_balances::Event::Endowed {
                account: AccountKeyring::Dave.to_account_id(),
                free_balance: 1_000
            })
        );
        assert_eq!(
            System::events()[3].event,
            Event::UniversalDividend(pallet_universal_dividend::Event::UdsAutoPaidAtRemoval {
                count: 1,
                total: 1_000,
                who: AccountKeyring::Dave.to_account_id(),
            })
        );
        assert_eq!(
            System::events()[4].event,
            Event::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 4 })
        );

        // Verify state
        assert!(Identity::identity(4).is_none());
        assert_eq!(
            Balances::free_balance(AccountKeyring::Dave.to_account_id()),
            1_000
        );
    });
}

#[test]
fn test_remove_smith_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            3,
            None
        ));
        let events = System::events();
        assert_eq!(events.len(), 4);
        assert_eq!(
            System::events()[0].event,
            Event::SmithsMembership(pallet_membership::Event::MembershipRevoked(3))
        );
        assert_eq!(
            System::events()[1].event,
            Event::AuthorityMembers(pallet_authority_members::Event::MemberRemoved(3))
        );
        assert_eq!(
            System::events()[2].event,
            Event::Membership(pallet_membership::Event::MembershipRevoked(3))
        );
        assert_eq!(
            System::events()[3].event,
            Event::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 3 })
        );
        //println!("{:#?}", events);
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
            let events = System::events();
            //println!("{:#?}", events);
            assert_eq!(events.len(), 3);
            assert_eq!(
                System::events()[0].event,
                Event::System(frame_system::Event::NewAccount {
                    account: AccountKeyring::Eve.to_account_id(),
                })
            );
            assert_eq!(
                System::events()[1].event,
                Event::Balances(pallet_balances::Event::Endowed {
                    account: AccountKeyring::Eve.to_account_id(),
                    free_balance: 400,
                })
            );
            assert_eq!(
                System::events()[2].event,
                Event::Balances(pallet_balances::Event::Transfer {
                    from: AccountKeyring::Alice.to_account_id(),
                    to: AccountKeyring::Eve.to_account_id(),
                    amount: 400,
                })
            );

            // At next block, the new account must be reaped because its balance is not sufficient
            // to pay the "new account tax"
            run_to_block(3);
            let events = System::events();
            //println!("{:#?}", events);
            assert_eq!(events.len(), 3);
            assert_eq!(
                System::events()[0].event,
                Event::Account(pallet_duniter_account::Event::ForceDestroy {
                    who: AccountKeyring::Eve.to_account_id(),
                    balance: 400,
                })
            );
            assert_eq!(
                System::events()[1].event,
                Event::Balances(pallet_balances::Event::Deposit {
                    who: Treasury::account_id(),
                    amount: 400,
                })
            );
            assert_eq!(
                System::events()[2].event,
                Event::Treasury(pallet_treasury::Event::Deposit { value: 400 })
            );
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
            System::assert_has_event(Event::System(frame_system::Event::NewAccount {
                account: AccountKeyring::Eve.to_account_id(),
            }));
            System::assert_has_event(Event::Balances(pallet_balances::Event::Endowed {
                account: AccountKeyring::Eve.to_account_id(),
                free_balance: 500,
            }));
            System::assert_has_event(Event::Balances(pallet_balances::Event::Transfer {
                from: AccountKeyring::Alice.to_account_id(),
                to: AccountKeyring::Eve.to_account_id(),
                amount: 500,
            }));

            // At next block, the new account must be created,
            // and new account tax should be collected and deposited in the treasury
            run_to_block(3);
            let events = System::events();
            println!("{:#?}", events);
            assert_eq!(events.len(), 3);
            assert_eq!(
                System::events()[0].event,
                Event::Balances(pallet_balances::Event::Withdraw {
                    who: AccountKeyring::Eve.to_account_id(),
                    amount: 300,
                })
            );
            assert_eq!(
                System::events()[1].event,
                Event::Balances(pallet_balances::Event::Deposit {
                    who: Treasury::account_id(),
                    amount: 300,
                })
            );
            assert_eq!(
                System::events()[2].event,
                Event::Treasury(pallet_treasury::Event::Deposit { value: 300 })
            );
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

            // The new member should have first_eligible_ud equal to one
            assert!(Identity::identity(5).is_some());
            assert_eq!(
                Identity::identity(5).unwrap().data,
                IdtyData {
                    first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::from(3),
                }
            );
        });
}
