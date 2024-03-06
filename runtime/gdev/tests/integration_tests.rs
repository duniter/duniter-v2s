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
use frame_support::traits::StoredMap;
use frame_support::traits::{Get, PalletInfo, StorageInfo, StorageInfoTrait};
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_support::{StorageHasher, Twox128};
use gdev_runtime::*;
use pallet_identity::{RevocationPayload, REVOCATION_PAYLOAD_PREFIX};
use pallet_membership::MembershipRemovalReason;
use pallet_smith_members::{SmithMeta, SmithStatus};
use sp_core::Encode;
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

/// test initial state
///
/// in order to make sure that it does not change accidentally
#[test]
fn test_initial_state() {
    ExtBuilder::new(1, 2, 3)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 1_000),
            (AccountKeyring::Bob.to_account_id(), 200),
            (AccountKeyring::Charlie.to_account_id(), 100), // below ED allowed for identities
            (AccountKeyring::Dave.to_account_id(), 900),
        ])
        .build()
        .execute_with(|| {
            run_to_block(1);

            assert_eq!(
                Balances::free_balance(AccountKeyring::Alice.to_account_id()),
                1_000
            );
            assert_eq!(
                Balances::free_balance(AccountKeyring::Bob.to_account_id()),
                200
            );
            assert_eq!(
                Balances::free_balance(AccountKeyring::Charlie.to_account_id()),
                100
            );
            assert_eq!(
                Balances::free_balance(AccountKeyring::Dave.to_account_id()),
                900
            );
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                0
            );
            // total issuance and monetary mass should be coherent
            assert_eq!(Balances::total_issuance(), 2_200);
            assert_eq!(
                pallet_universal_dividend::MonetaryMass::<Runtime>::get(),
                2_200
            );
        });
}

/// test total issuance against monetary mass
/// the monetary mass represents the claimable monetary mass
/// the total issuance represents the actually claimed currency
#[test]
fn test_total_issuance_vs_monetary_mass() {
    ExtBuilder::new(1, 2, 3)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 2000),
            (AccountKeyring::Bob.to_account_id(), 1000),
            (AccountKeyring::Charlie.to_account_id(), 0),
        ])
        .build()
        .execute_with(|| {
            let ud_creation_period =
                <Runtime as pallet_universal_dividend::Config>::UdCreationPeriod::get();
            assert_eq!(ud_creation_period, 60_000); // this is 10 blocks × 6 seconds in milliseconds

            run_to_block(1);
            // total issuance and monetary mass should be coherent
            assert_eq!(Balances::total_issuance(), 3000);
            assert_eq!(
                pallet_universal_dividend::MonetaryMass::<Runtime>::get(),
                3000
            );
            // first UD creation
            run_to_block(11);
            assert_eq!(Balances::total_issuance(), 3000);
            assert_eq!(
                pallet_universal_dividend::MonetaryMass::<Runtime>::get(),
                6000
            );
            // Alice claims her UD
            assert_ok!(UniversalDividend::claim_uds(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into()
            ));
            assert_eq!(Balances::total_issuance(), 4000);
            assert_eq!(
                pallet_universal_dividend::MonetaryMass::<Runtime>::get(),
                6000
            );
            // second UD creation
            run_to_block(21);
            // Bob claims his 2 UDs
            assert_ok!(UniversalDividend::claim_uds(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into()
            ));
            assert_eq!(Balances::total_issuance(), 6000);
            assert_eq!(
                pallet_universal_dividend::MonetaryMass::<Runtime>::get(),
                9000
            );
        });
}

/// test identity go below ED
#[test]
fn test_identity_below_ed() {
    ExtBuilder::new(1, 1, 1)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 900)])
        .build()
        .execute_with(|| {
            run_to_block(1);
            // new behavior : nobody is able to go below ED without killing the account
            // a transfer below ED will lead to frozen token error
            assert_noop!(
                Balances::transfer_allow_death(
                    frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                    MultiAddress::Id(AccountKeyring::Bob.to_account_id()),
                    850
                ),
                sp_runtime::TokenError::Frozen
            );
            // // Old behavior below
            // // Should be able to go below existential deposit, loose dust, and still not die
            // assert_ok!(Balances::transfer(
            //     frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
            //     MultiAddress::Id(AccountKeyring::Bob.to_account_id()),
            //     800
            // ));
            // assert_eq!(
            //     Balances::free_balance(AccountKeyring::Alice.to_account_id()),
            //     0
            // );
            // assert_eq!(
            //     Balances::free_balance(AccountKeyring::Bob.to_account_id()),
            //     800
            // );
            // // since alice is identity, her account should not be killed even she lost currency below ED
            // System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Transfer {
            //     from: AccountKeyring::Alice.to_account_id(),
            //     to: AccountKeyring::Bob.to_account_id(),
            //     amount: 800,
            // }));
            // System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::DustLost {
            //     account: AccountKeyring::Alice.to_account_id(),
            //     amount: 100,
            // }));
            // System::assert_has_event(RuntimeEvent::System(frame_system::Event::NewAccount {
            //     account: AccountKeyring::Bob.to_account_id(),
            // }));
            // System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Endowed {
            //     account: AccountKeyring::Bob.to_account_id(),
            //     free_balance: 800,
            // }));
        })
}

/// test session change
// session duration is set to 25 blocks
// this is to test that mock code behaves well
#[test]
fn test_session_change() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        assert_eq!(<Runtime as pallet_babe::Config>::EpochDuration::get(), 25);
        assert_eq!(Session::current_index(), 0);
        assert_eq!(SmithMembers::current_session(), 0);
        assert_eq!(Babe::epoch_index(), 0);
        assert_eq!(Babe::current_epoch_start(), 0u64);
        run_to_block(2);
        assert_eq!(Session::current_index(), 0);
        assert_eq!(Babe::epoch_index(), 0);
        run_to_block(24);
        assert_eq!(Session::current_index(), 0);
        assert_eq!(SmithMembers::current_session(), 0);
        assert_eq!(Babe::epoch_index(), 0);
        run_to_block(25);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(SmithMembers::current_session(), 1);
        assert_eq!(Babe::epoch_index(), 1);
        assert_eq!(Babe::current_epoch_start(), 25u64);
        run_to_block(26);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(SmithMembers::current_session(), 1);
        assert_eq!(Babe::epoch_index(), 1);
        run_to_block(50);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(SmithMembers::current_session(), 2);
        assert_eq!(Babe::epoch_index(), 2);
        assert_eq!(Babe::current_epoch_start(), 50u64);
        run_to_block(51);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(SmithMembers::current_session(), 2);
        assert_eq!(Babe::epoch_index(), 2);
        run_to_block(52);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(SmithMembers::current_session(), 2);
        assert_eq!(Babe::epoch_index(), 2);
        run_to_block(60);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(SmithMembers::current_session(), 2);
        assert_eq!(Babe::epoch_index(), 2);
        assert_eq!(Babe::current_epoch_start(), 50u64);
        run_to_block(75);
        assert_eq!(Session::current_index(), 3);
        assert_eq!(SmithMembers::current_session(), 3);
        assert_eq!(Babe::epoch_index(), 3);
        run_to_block(100);
        assert_eq!(Session::current_index(), 4);
        assert_eq!(SmithMembers::current_session(), 4);
        assert_eq!(Babe::epoch_index(), 4);
    })
}

/// test calling do_remove_identity
#[test]
fn test_remove_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);
        // remove the identity
        Identity::do_remove_identity(4, pallet_identity::RemovalReason::Root);
        // // membership removal is no more automatic
        // System::assert_has_event(RuntimeEvent::Membership(
        //     pallet_membership::Event::MembershipRemoved {
        //         member: 4,
        //         reason: MembershipRemovalReason::System,
        //     },
        // ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRemoved {
                idty_index: 4,
                reason: pallet_identity::RemovalReason::Root,
            },
        ));
        // since Dave does not have ED, his account is killed
        System::assert_has_event(RuntimeEvent::System(frame_system::Event::KilledAccount {
            account: AccountKeyring::Dave.to_account_id(),
        }));
    });
}

/// test identity is "validated" (= membership is claimed) when distance is evaluated positively
#[test]
fn test_validate_identity_when_claim() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Eve.to_account_id(), 2000),
            (AccountKeyring::Ferdie.to_account_id(), 1000),
        ])
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
            assert_ok!(Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                5
            ));
            assert_ok!(Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into(),
                5
            ));

            // eve request distance evaluation for herself
            assert_ok!(Distance::request_distance_evaluation(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            ));

            // Pass 2nd evaluation period
            let eval_period: u32 = <Runtime as pallet_distance::Config>::EvaluationPeriod::get();
            run_to_block(2 * eval_period);
            // simulate an evaluation published by smith Alice
            assert_ok!(Distance::force_update_evaluation(
                frame_system::RawOrigin::Root.into(),
                AccountKeyring::Alice.to_account_id(),
                pallet_distance::ComputationResult {
                    distances: vec![Perbill::one()],
                }
            ));
            // Pass 3rd evaluation period
            run_to_block(3 * eval_period);
            System::assert_has_event(RuntimeEvent::Distance(
                pallet_distance::Event::EvaluatedValid { idty_index: 5 },
            ));

            // eve can not claim her membership manually because it is done automatically
            // the following call does not exist anymore
            // assert_noop!(
            //     Membership::claim_membership(
            //         frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            //     ),
            //     pallet_membership::Error::<Runtime>::AlreadyMember
            // );

            // println!("{:?}", System::events());
            System::assert_has_event(RuntimeEvent::Membership(
                pallet_membership::Event::MembershipAdded {
                    member: 5,
                    expire_on: 3 * eval_period
                        + <Runtime as pallet_membership::Config>::MembershipPeriod::get(),
                },
            ));
        });
}

/// test identity creation workflow
// with distance requested by last certifier
#[test]
fn test_identity_creation_workflow() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Charlie.to_account_id(), 10_000), // necessary for evalation distance reserve
            (AccountKeyring::Eve.to_account_id(), 2_000),
            (AccountKeyring::Ferdie.to_account_id(), 1_000),
        ])
        .build()
        .execute_with(|| {
            run_to_block(1);
            // alice create identity for Eve
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));
            assert_eq!(
                Identity::identity(5),
                Some(pallet_identity::IdtyValue {
                    data: Default::default(),
                    next_creatable_identity_on: 0u32,
                    old_owner_key: None,
                    owner_key: AccountKeyring::Eve.to_account_id(),
                    next_scheduled: 1 + 40,
                    status: pallet_identity::IdtyStatus::Unconfirmed,
                })
            );
            run_to_block(2);
            // eve confirms her identity
            assert_ok!(Identity::confirm_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                "Eeeeeveeeee".into(),
            ));
            assert_eq!(
                Identity::identity(5),
                Some(pallet_identity::IdtyValue {
                    data: Default::default(),
                    next_creatable_identity_on: 0u32,
                    old_owner_key: None,
                    owner_key: AccountKeyring::Eve.to_account_id(),
                    next_scheduled: 2 + 876600,
                    status: pallet_identity::IdtyStatus::Unvalidated,
                })
            );
            run_to_block(3);
            // eve gets certified by bob and charlie
            assert_ok!(Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                5
            ));
            assert_ok!(Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into(),
                5
            ));
            // charlie also request distance evaluation for eve
            // (could be done in batch)
            assert_ok!(Distance::request_distance_evaluation_for(
                frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into(),
                5
            ));
            // then the evaluation is pending
            assert_eq!(
                Distance::pending_evaluation_request(5),
                Some(AccountKeyring::Charlie.to_account_id(),)
            );

            // Pass 2nd evaluation period
            let eval_period: u32 = <Runtime as pallet_distance::Config>::EvaluationPeriod::get();
            run_to_block(2 * eval_period);
            // simulate evaluation published by smith Alice
            assert_ok!(Distance::force_update_evaluation(
                frame_system::RawOrigin::Root.into(),
                AccountKeyring::Alice.to_account_id(),
                pallet_distance::ComputationResult {
                    distances: vec![Perbill::one()],
                }
            ));
            // Pass 3rd evaluation period
            run_to_block(3 * eval_period);

            // eve should not even have to claim her membership
            System::assert_has_event(RuntimeEvent::Membership(
                pallet_membership::Event::MembershipAdded {
                    member: 5,
                    expire_on: 3 * eval_period
                        + <Runtime as pallet_membership::Config>::MembershipPeriod::get(),
                },
            ));

            // test state coherence
            // block time is 6_000 ms
            // ud creation period is 60_000 ms ~ 10 blocks
            // first ud is at 24_000 ms ~ 4 blocks
            // at current block this is UD number current_block/10 + 1
            let first_eligible = ((3 * eval_period) / 10 + 1) as u16;
            assert_eq!(
                Identity::identity(5),
                Some(pallet_identity::IdtyValue {
                    data: IdtyData {
                        first_eligible_ud: pallet_universal_dividend::FirstEligibleUd(Some(
                            sp_std::num::NonZeroU16::new(first_eligible).unwrap()
                        ))
                    },
                    next_creatable_identity_on: 0u32,
                    old_owner_key: None,
                    owner_key: AccountKeyring::Eve.to_account_id(),
                    next_scheduled: 0,
                    status: pallet_identity::IdtyStatus::Member,
                })
            );

            run_to_block(84);
            System::assert_has_event(RuntimeEvent::UniversalDividend(
                pallet_universal_dividend::Event::NewUdCreated {
                    amount: 1000,
                    index: 9,
                    monetary_mass: 49_000 + (10 - first_eligible as u64) * 1_000, // 13_000 (initial) + 4 * 1000 * 9 (produced) + (10-first_eligible)*1_000
                    members_count: 5,
                },
            ));
            assert_ok!(UniversalDividend::claim_uds(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            ));
            System::assert_has_event(RuntimeEvent::UniversalDividend(
                pallet_universal_dividend::Event::UdsClaimed {
                    count: (10 - first_eligible),
                    total: (10 - first_eligible as u64) * 1_000,
                    who: AccountKeyring::Eve.to_account_id(),
                },
            ));
        });
}

/// an identity should not be able to add cert
/// when its membership is suspended
#[test]
fn test_can_not_issue_cert_when_membership_lost() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        // expire Bob membership
        Membership::do_remove_membership(2, MembershipRemovalReason::System);
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 2,
                reason: MembershipRemovalReason::System,
            },
        ));

        // Bob can not issue a certification
        assert_noop!(
            Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                3,
            ),
            pallet_duniter_wot::Error::<gdev_runtime::Runtime>::IssuerNotMember
        );
    });
}

/// test membership expiry
#[test]
fn test_membership_expiry() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(100);
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 1,
                reason: MembershipRemovalReason::Expired,
            },
        ));
        // membership expiry should not trigger identity removal
        assert!(Identity::identity(1).is_some());
    });
}

// TODO: use change_parameter to change autorevocation period
#[test]
#[ignore = "long to go to autorevocation period"]
fn test_membership_expiry_with_identity_removal() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(100);

        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 4,
                reason: MembershipRemovalReason::Expired,
            },
        ));

        // Trigger pending membership expiry
        run_to_block(100 + <Runtime as pallet_identity::Config>::AutorevocationPeriod::get());

        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRevoked {
                idty_index: 4,
                reason: pallet_identity::RevocationReason::Expired,
            },
        ));
    });
}

/// test membership renewal
#[test]
fn test_membership_renewal() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 2000)])
        .build()
        .execute_with(|| {
            // can not renew membership immediately
            assert_noop!(
                Distance::request_distance_evaluation(
                    frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                ),
                pallet_duniter_wot::Error::<Runtime>::MembershipRenewalPeriodNotRespected,
            );

            // but ok after waiting 10 blocks delay
            run_to_block(11);
            assert_ok!(Distance::request_distance_evaluation(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
            ));

            // Pass 3rd evaluation period
            let eval_period: u32 = <Runtime as pallet_distance::Config>::EvaluationPeriod::get();
            run_to_block(3 * eval_period);
            assert_ok!(Distance::force_update_evaluation(
                frame_system::RawOrigin::Root.into(),
                AccountKeyring::Alice.to_account_id(),
                pallet_distance::ComputationResult {
                    distances: vec![Perbill::one()],
                }
            ));
            // Pass to 4th evaluation period
            run_to_block(4 * eval_period);
            System::assert_has_event(RuntimeEvent::Membership(
                pallet_membership::Event::MembershipRenewed {
                    member: 1,
                    expire_on: 4 * eval_period
                        + <Runtime as pallet_membership::Config>::MembershipPeriod::get(),
                },
            ));

            run_to_block(4 * eval_period + 1);
            // not possible to renew manually
            // can not ask renewal when period is not respected
            assert_noop!(
                Distance::request_distance_evaluation(
                    frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                ),
                pallet_duniter_wot::Error::<Runtime>::MembershipRenewalPeriodNotRespected,
            );

            // should expire at block 3nd EvaluationPeriod + MembershipPeriod
            run_to_block(
                4 * eval_period + <Runtime as pallet_membership::Config>::MembershipPeriod::get(),
            );
            System::assert_has_event(RuntimeEvent::Membership(
                pallet_membership::Event::MembershipRemoved {
                    member: 1,
                    reason: MembershipRemovalReason::Expired,
                },
            ));
        });
}

// test that UD are auto claimed when identity is revoked
#[test]
fn test_revoke_identity_after_one_ud() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        //println!("UdCreationPeriod={}", <Runtime as pallet_universal_dividend::Config>::UdCreationPeriod::get());
        run_to_block(
            (<Runtime as pallet_universal_dividend::Config>::UdCreationPeriod::get()
                / <Runtime as pallet_babe::Config>::ExpectedBlockTime::get()
                + 1) as u32,
        );

        // before UD, dave has 0 (initial amount)
        run_to_block(1);
        assert_eq!(
            Balances::free_balance(AccountKeyring::Dave.to_account_id()),
            0
        );

        // go after UD creation block
        run_to_block(
            (<Runtime as pallet_universal_dividend::Config>::UdCreationPeriod::get()
                / <Runtime as pallet_babe::Config>::ExpectedBlockTime::get()
                + 1) as u32,
        );
        // revoke identity
        Identity::do_revoke_identity(4, pallet_identity::RevocationReason::Root);

        // Verify events
        // universal dividend was automatically paid to dave
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::UdsAutoPaid {
                count: 1,
                total: 1_000,
                who: AccountKeyring::Dave.to_account_id(),
            },
        ));
        // dave account actually received this UD
        System::assert_has_event(RuntimeEvent::Balances(pallet_balances::Event::Deposit {
            who: AccountKeyring::Dave.to_account_id(),
            amount: 1_000,
        }));
        // membership and identity were actually removed
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 4,
                reason: MembershipRemovalReason::Revoked,
            },
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRevoked {
                idty_index: 4,
                reason: pallet_identity::RevocationReason::Root,
            },
        ));

        assert!(Identity::identity(4).is_some()); // identity still exists, but its status is revoked
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
        // UD are created every 10 blocks from block 4
        run_to_block(4);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::NewUdCreated {
                amount: 1000,
                index: 1,
                monetary_mass: 4_000, // 0 (initial) + 4 * 1000 (produced)
                members_count: 4,
            },
        ));
        // UD not claimed, still initial balance to initial 0
        assert_eq!(
            Balances::free_balance(AccountKeyring::Alice.to_account_id()),
            0
        );

        run_to_block(13);
        // alice identity expires
        Membership::do_remove_membership(1, MembershipRemovalReason::System);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::UdsAutoPaid {
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
        run_to_block(14);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::NewUdCreated {
                amount: 1000,
                index: 2,
                monetary_mass: 7_000, // 4000 + 3 × 1000
                members_count: 3,     // alice is not member at this UD
            },
        ));

        // alice claims back her membership through distance evaluation
        assert_ok!(Distance::force_valid_distance_status(
            frame_system::RawOrigin::Root.into(),
            1,
        ));
        // it can not be done manually
        // because the call does not exist anymore
        // assert_noop!(
        //     Membership::claim_membership(
        //         frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
        //     ),
        //     pallet_membership::Error::<Runtime>::AlreadyMember
        // );
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipAdded {
                member: 1,
                expire_on: 14 + <Runtime as pallet_membership::Config>::MembershipPeriod::get(),
            },
        ));

        // UD number 3
        run_to_block(24);
        System::assert_has_event(RuntimeEvent::UniversalDividend(
            pallet_universal_dividend::Event::NewUdCreated {
                amount: 1000,
                index: 3,
                monetary_mass: 11000, // 7000 + 4 × 1000
                members_count: 4,     // alice is member again at this UD
            },
        ));

        // one block later, alice claims her new UD
        run_to_block(25);
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

/// test when root revokes and identity, all membership should be deleted
#[test]
fn test_revoke_smith_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        Identity::do_revoke_identity(3, pallet_identity::RevocationReason::Root);
        // Verify events
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberRemoved { member: 3 },
        ));
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 3,
                reason: MembershipRemovalReason::Revoked,
            },
        ));
        System::assert_has_event(RuntimeEvent::Identity(
            pallet_identity::Event::IdtyRevoked {
                idty_index: 3,
                reason: pallet_identity::RevocationReason::Root,
            },
        ));
    });
}

#[test]
fn test_smith_certification() {
    // 3 smith (1. alice, 2. bob, 3. charlie)
    // 4 identities (4. dave)
    // no identity 5. eve
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);

        assert_noop!(
            SmithMembers::certify_smith(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                2
            ),
            pallet_smith_members::Error::<Runtime>::CertificationAlreadyExists
        );

        assert_noop!(
            SmithMembers::certify_smith(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                4
            ),
            pallet_smith_members::Error::<Runtime>::CertificationReceiverMustHaveBeenInvited
        );
    });
}

fn create_dummy_session_keys() -> gdev_runtime::opaque::SessionKeys {
    gdev_runtime::opaque::SessionKeys {
        grandpa: sp_core::ed25519::Public([0u8; 32]).into(),
        babe: sp_core::sr25519::Public([0u8; 32]).into(),
        im_online: sp_core::sr25519::Public([0u8; 32]).into(),
        authority_discovery: sp_core::sr25519::Public([0u8; 32]).into(),
    }
}

/// test the full process to join smith from main wot member to authority member
#[test]
fn test_smith_process() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Dave.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(1);

            let alice = AccountKeyring::Alice.to_account_id();
            let bob = AccountKeyring::Bob.to_account_id();
            let charlie = AccountKeyring::Charlie.to_account_id();
            let dave = AccountKeyring::Dave.to_account_id();
            let dummy_session_keys = create_dummy_session_keys();

            // Eve can not request smith membership because not member of the smith wot
            // no more membership request

            // Dave can request smith membership (currently optional)
            // no more membership request

            assert_ok!(SmithMembers::invite_smith(
                frame_system::RawOrigin::Signed(alice.clone()).into(),
                4
            ));
            assert_ok!(SmithMembers::accept_invitation(
                frame_system::RawOrigin::Signed(dave).into(),
            ));

            // Dave cannot (yet) set his session keys
            assert_err!(
                AuthorityMembers::set_session_keys(
                    frame_system::RawOrigin::Signed(AccountKeyring::Dave.to_account_id()).into(),
                    dummy_session_keys.clone()
                ),
                pallet_authority_members::Error::<Runtime>::NotMember
            );

            // Alice Bob and Charlie can certify Dave
            assert_ok!(SmithMembers::certify_smith(
                frame_system::RawOrigin::Signed(alice.clone()).into(),
                4
            ));
            assert_ok!(SmithMembers::certify_smith(
                frame_system::RawOrigin::Signed(bob.clone()).into(),
                4
            ));
            assert_ok!(SmithMembers::certify_smith(
                frame_system::RawOrigin::Signed(charlie.clone()).into(),
                4
            ));

            // with these three smith certs, Dave has become smith
            // Dave is then member of the smith wot
            assert_eq!(
                SmithMembers::smiths(4),
                Some(pallet_smith_members::SmithMeta {
                    status: SmithStatus::Smith,
                    expires_on: Some(48),
                    issued_certs: vec![],
                    received_certs: vec![1, 2, 3],
                })
            );

            // Dave can set his (dummy) session keys
            assert_ok!(AuthorityMembers::set_session_keys(
                frame_system::RawOrigin::Signed(AccountKeyring::Dave.to_account_id()).into(),
                dummy_session_keys
            ));

            // Dave can go online
            assert_ok!(AuthorityMembers::go_online(
                frame_system::RawOrigin::Signed(AccountKeyring::Dave.to_account_id()).into(),
            ));
        })
}

/// test new account creation
#[test]
fn test_create_new_account() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(2);
            assert_eq!(Balances::free_balance(Treasury::account_id()), 100);

            // Should be able to transfer 5 units to a new account
            assert_ok!(Balances::transfer_allow_death(
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

            // The new account must be created immediately
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                500
            );
            // 100 initial + no deposit (there is no account creation fee)
            assert_eq!(Balances::free_balance(Treasury::account_id()), 100);

            // can remove an account using transfer
            assert_ok!(Balances::transfer_allow_death(
                frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Alice.to_account_id()),
                500
            ));
            // Account reaped
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                0
            );
            assert_eq!(
                frame_system::Pallet::<Runtime>::get(&AccountKeyring::Eve.to_account_id()),
                pallet_duniter_account::AccountData::default()
            );
            System::assert_has_event(RuntimeEvent::System(frame_system::Event::KilledAccount {
                account: AccountKeyring::Eve.to_account_id(),
            }));
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
            assert_ok!(Balances::transfer_allow_death(
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
        });
}

#[test]
fn test_create_new_idty_without_founds() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            run_to_block(2);
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                0
            );

            // Should be able to create an identity without founds
            assert_ok!(Identity::create_identity(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                AccountKeyring::Eve.to_account_id(),
            ));
            System::assert_has_event(RuntimeEvent::Identity(
                pallet_identity::Event::IdtyCreated {
                    idty_index: 5,
                    owner_key: AccountKeyring::Eve.to_account_id(),
                },
            ));

            // At next block, nothing should be preleved
            run_to_block(3);
            let events = System::events();
            assert_eq!(events.len(), 0);

            // Deposit some founds on the identity account
            assert_ok!(Balances::transfer_allow_death(
                frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
                MultiAddress::Id(AccountKeyring::Eve.to_account_id()),
                200
            ));

            // At next block, nothing should be preleved
            run_to_block(4);
            assert_eq!(
                Balances::free_balance(AccountKeyring::Eve.to_account_id()),
                200
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
            assert_ok!(Balances::transfer_allow_death(
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

            // At next block, Bob should be able to certify the new identity
            run_to_block(23);
            assert_ok!(Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                5,
            ));
            // valid distance status should trigger identity validation
            assert_ok!(Distance::force_valid_distance_status(
                frame_system::RawOrigin::Root.into(),
                5,
            ));
            // and it is not possible to call it manually
            // because the call does not exist anymore
            // assert_noop!(
            //     Membership::claim_membership(
            //         frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            //     ),
            //     pallet_membership::Error::<Runtime>::AlreadyMember
            // );

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
/// even when the method used is membership claim
#[test]
fn test_claim_memberhsip_after_few_uds() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![
            (AccountKeyring::Alice.to_account_id(), 1_000),
            (AccountKeyring::Bob.to_account_id(), 1_000),
            (AccountKeyring::Charlie.to_account_id(), 1_000),
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
            assert_ok!(Certification::add_cert(
                frame_system::RawOrigin::Signed(AccountKeyring::Bob.to_account_id()).into(),
                5,
            ));

            // eve membership should be able to be claimed through distance evaluation
            assert_ok!(Distance::force_valid_distance_status(
                frame_system::RawOrigin::Root.into(),
                5,
            ));
            // but not manually
            // because the call does not exist
            // assert_noop!(
            //     Membership::claim_membership(
            //         frame_system::RawOrigin::Signed(AccountKeyring::Eve.to_account_id()).into(),
            //     ),
            //     pallet_membership::Error::<Runtime>::AlreadyMember
            // );

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

/// test linking account to identity
#[test]
fn test_link_account() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Alice.to_account_id(), 8888)])
        .build()
        .execute_with(|| {
            let genesis_hash = System::block_hash(0);
            let alice = AccountKeyring::Alice.to_account_id();
            let ferdie = AccountKeyring::Ferdie.to_account_id();
            let payload = (b"link", genesis_hash, 1u32, ferdie.clone()).encode();
            let signature = AccountKeyring::Ferdie.sign(&payload);

            // Ferdie's account cannot be linked to Alice identity because the account does not exist
            assert_noop!(
                Identity::link_account(
                    frame_system::RawOrigin::Signed(alice.clone()).into(),
                    ferdie.clone(),
                    signature.clone().into()
                ),
                pallet_identity::Error::<gdev_runtime::Runtime>::AccountNotExist
            );

            assert_ok!(Balances::transfer_allow_death(
                frame_system::RawOrigin::Signed(alice.clone()).into(),
                MultiAddress::Id(ferdie.clone()),
                1_000
            ));
            // Ferdie's account can be linked to Alice identity
            assert_ok!(Identity::link_account(
                frame_system::RawOrigin::Signed(alice).into(),
                ferdie,
                signature.into()
            ));
        })
}

/// test change owner key
#[test]
fn test_change_owner_key() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        let genesis_hash = System::block_hash(0);
        let charlie = AccountKeyring::Charlie.to_account_id();
        let ferdie = AccountKeyring::Ferdie.to_account_id();
        let payload = (b"icok", genesis_hash, 3u32, charlie.clone()).encode();
        let signature = AccountKeyring::Ferdie.sign(&payload);

        SmithMembers::on_smith_goes_offline(3);
        // Charlie is now offline smith
        assert_eq!(
            SmithMembers::smiths(3),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(48),
                issued_certs: vec![1, 2],
                received_certs: vec![1, 2]
            })
        );

        assert_eq!(
            frame_system::Pallet::<Runtime>::get(&charlie).linked_idty,
            Some(3)
        );
        assert_eq!(
            frame_system::Pallet::<Runtime>::get(&ferdie).linked_idty,
            None
        );
        // Dave can change his owner key to Ferdie's
        assert_ok!(Identity::change_owner_key(
            frame_system::RawOrigin::Signed(charlie.clone()).into(),
            ferdie.clone(),
            signature.into()
        ));
        assert_eq!(
            frame_system::Pallet::<Runtime>::get(&ferdie).linked_idty,
            Some(3)
        );

        // Charlie is still an offline smith
        assert_eq!(
            SmithMembers::smiths(3),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(48),
                issued_certs: vec![1, 2],
                received_certs: vec![1, 2]
            })
        );

        // Ferdie can set its session_keys and go online
        frame_system::Pallet::<Runtime>::inc_providers(&ferdie);
        assert_ok!(AuthorityMembers::set_session_keys(
            frame_system::RawOrigin::Signed(AccountKeyring::Ferdie.to_account_id()).into(),
            create_dummy_session_keys()
        ));
        assert_ok!(AuthorityMembers::go_online(
            frame_system::RawOrigin::Signed(AccountKeyring::Ferdie.to_account_id()).into()
        ));

        // Charlie is still an offline smith
        assert_eq!(
            SmithMembers::smiths(3),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(48),
                issued_certs: vec![1, 2],
                received_certs: vec![1, 2]
            })
        );

        run_to_block(25);

        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::IncomingAuthorities { members: vec![3] },
        ));
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::OutgoingAuthorities { members: vec![1] },
        ));

        // "Charlie" (idty 3) is now online because its identity is mapped to Ferdies's key
        assert_eq!(
            SmithMembers::smiths(3),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: None,
                issued_certs: vec![1, 2],
                received_certs: vec![1, 2]
            })
        );
    })
}

/// members of the smith subwot can revoke their identity
#[test]
fn test_smith_member_can_revoke_its_idty() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        // Charlie goes online
        frame_system::Pallet::<Runtime>::inc_providers(&AccountKeyring::Charlie.to_account_id());
        assert_ok!(AuthorityMembers::set_session_keys(
            frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into(),
            create_dummy_session_keys()
        ));
        assert_ok!(AuthorityMembers::go_online(
            frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into()
        ));

        run_to_block(25);

        // Charlie is in the authority members
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::IncomingAuthorities { members: vec![3] },
        ));
        // Charlie is not going out
        assert!(!pallet_authority_members::OutgoingAuthorities::<Runtime>::get().contains(&3));

        let revocation_payload = RevocationPayload {
            idty_index: 3u32,
            genesis_hash: System::block_hash(0),
        };
        let signature =
            AccountKeyring::Charlie.sign(&(REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode());

        assert_ok!(Identity::revoke_identity(
            frame_system::RawOrigin::Signed(AccountKeyring::Charlie.to_account_id()).into(),
            3,
            AccountKeyring::Charlie.to_account_id(),
            signature.into()
        ));
        // membership should be removed
        System::assert_has_event(RuntimeEvent::Membership(
            pallet_membership::Event::MembershipRemoved {
                member: 3,
                reason: MembershipRemovalReason::Revoked,
            },
        ));
        // smith membership should be removed as well
        System::assert_has_event(RuntimeEvent::SmithMembers(
            pallet_smith_members::Event::SmithMembershipRemoved { idty_index: 3 },
        ));
        System::assert_has_event(RuntimeEvent::SmithMembers(
            pallet_smith_members::Event::SmithCertRemoved {
                receiver: 3,
                issuer: 1,
            },
        ));
        System::assert_has_event(RuntimeEvent::SmithMembers(
            pallet_smith_members::Event::SmithCertRemoved {
                receiver: 3,
                issuer: 2,
            },
        ));
        // Now Charlie is going out
        assert!(pallet_authority_members::OutgoingAuthorities::<Runtime>::get().contains(&3));
    });
}

/// test genesis account of identity is linked to identity
// (and account without identity is not linked)
#[test]
fn test_genesis_account_of_identity_linked() {
    ExtBuilder::new(1, 3, 4)
        .with_initial_balances(vec![(AccountKeyring::Eve.to_account_id(), 8888)])
        .build()
        .execute_with(|| {
            // Alice account
            let account_id = AccountKeyring::Alice.to_account_id();
            // Alice identity index is 1
            assert_eq!(Identity::identity_index_of(&account_id), Some(1));
            // get account data
            let account_data = frame_system::Pallet::<Runtime>::get(&account_id);
            assert_eq!(account_data.linked_idty, Some(1));
            // Eve is not member, her account has no linked identity
            assert_eq!(
                frame_system::Pallet::<Runtime>::get(&AccountKeyring::Eve.to_account_id())
                    .linked_idty,
                None
            );
        })
}

/// test unlink identity from account
#[test]
fn test_unlink_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        let alice_account = AccountKeyring::Alice.to_account_id();
        // check that Alice is 1
        assert_eq!(Identity::identity_index_of(&alice_account), Some(1));

        // Alice can unlink her identity from her account
        assert_ok!(Account::unlink_identity(
            frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
        ));

        // Alice account has been unlinked
        assert_eq!(
            frame_system::Pallet::<Runtime>::get(&alice_account).linked_idty,
            None
        );
    })
}

/// test that the account of a newly created identity is linked to the identity
#[test]
fn test_new_account_linked() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        let eve_account = AccountKeyring::Eve.to_account_id();
        assert_eq!(
            frame_system::Pallet::<Runtime>::get(&eve_account).linked_idty,
            None
        );
        // Alice creates identity for Eve
        assert_ok!(Identity::create_identity(
            frame_system::RawOrigin::Signed(AccountKeyring::Alice.to_account_id()).into(),
            eve_account.clone(),
        ));
        // then eve account should be linked to her identity
        assert_eq!(
            frame_system::Pallet::<Runtime>::get(&eve_account).linked_idty,
            Some(5)
        );
    })
}

/// test killed account
// The only way to kill an account is to kill  the identity
// and transfer all funds.
#[test]
fn test_killed_account() {
    ExtBuilder::new(1, 2, 4)
        .with_initial_balances(vec![(AccountKeyring::Bob.to_account_id(), 1_000)])
        .build()
        .execute_with(|| {
            let alice_account = AccountKeyring::Alice.to_account_id();
            let bob_account = AccountKeyring::Bob.to_account_id();
            // check that Alice is 1 and Bob 2
            assert_eq!(Identity::identity_index_of(&alice_account), Some(1));
            assert_eq!(Identity::identity_index_of(&bob_account), Some(2));

            let _ = Identity::do_remove_identity(2, pallet_identity::RemovalReason::Revoked);
            assert_eq!(
                frame_system::Pallet::<Runtime>::get(&bob_account).linked_idty,
                Some(2)
            );
            assert_ok!(Balances::transfer_all(
                frame_system::RawOrigin::Signed(bob_account.clone()).into(),
                sp_runtime::MultiAddress::Id(alice_account.clone()),
                false
            ));

            // Bob account should have been reaped
            assert_eq!(
                frame_system::Pallet::<Runtime>::get(&bob_account),
                pallet_duniter_account::AccountData::default()
            );
        })
}
