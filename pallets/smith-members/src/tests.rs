// Copyright 2021-2023 Axiom-Team
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

use super::*;
use crate::mock::{new_test_ext, run_to_block, Runtime, RuntimeEvent, RuntimeOrigin, System};
use frame_support::{assert_err, assert_ok};

use crate::SmithStatus::{Excluded, Invited, Pending, Smith};
#[cfg(test)]
use maplit::btreemap;
use pallet_authority_members::OnNewSession;

#[test]
fn process_to_become_a_smith_and_lose_it() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_online(1);
        Pallet::<Runtime>::on_smith_goes_online(2);
        // Events cannot be recorded on genesis
        run_to_block(1);
        // State before
        assert_eq!(Smiths::<Runtime>::get(5), None);
        // Try to invite
        assert_ok!(Pallet::<Runtime>::invite_smith(RuntimeOrigin::signed(1), 5));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::InvitationSent {
            receiver: 5,
            issuer: 1,
        }));
        // Accept invitation
        assert_ok!(Pallet::<Runtime>::accept_invitation(RuntimeOrigin::signed(
            5
        )));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::InvitationAccepted {
            idty_index: 5,
        }));
        // State after
        assert_eq!(
            Smiths::<Runtime>::get(5).unwrap(),
            SmithMeta {
                status: SmithStatus::Pending,
                expires_on: Some(5),
                issued_certs: vec![],
                received_certs: vec![],
            }
        );
        // Then certification 1/2
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(1),
            5
        ));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertAdded {
            receiver: 5,
            issuer: 1,
        }));
        assert_eq!(
            Smiths::<Runtime>::get(5).unwrap(),
            SmithMeta {
                status: SmithStatus::Pending,
                expires_on: Some(5),
                issued_certs: vec![],
                received_certs: vec![1],
            }
        );
        // Then certification 2/2
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(2),
            5
        ));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertAdded {
            receiver: 5,
            issuer: 1,
        }));
        System::assert_has_event(RuntimeEvent::Smith(
            Event::<Runtime>::SmithMembershipAdded { idty_index: 5 },
        ));
        assert_eq!(
            Smiths::<Runtime>::get(5).unwrap(),
            SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(5),
                issued_certs: vec![],
                received_certs: vec![1, 2],
            }
        );
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_offline(1);
        Pallet::<Runtime>::on_smith_goes_offline(2);

        // On session 4 everything if fine
        Pallet::<Runtime>::on_new_session(4);
        assert!(Smiths::<Runtime>::get(1).is_some());
        assert!(Smiths::<Runtime>::get(2).is_some());
        assert!(Smiths::<Runtime>::get(5).is_some());
        // On session 5 no more smiths because of lack of activity
        Pallet::<Runtime>::on_new_session(5);
        System::assert_has_event(RuntimeEvent::Smith(
            Event::<Runtime>::SmithMembershipRemoved { idty_index: 1 },
        ));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 1,
            issuer: 2,
        }));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 1,
            issuer: 3,
        }));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 1,
            issuer: 4,
        }));
        System::assert_has_event(RuntimeEvent::Smith(
            Event::<Runtime>::SmithMembershipRemoved { idty_index: 2 },
        ));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 2,
            issuer: 3,
        }));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 2,
            issuer: 4,
        }));
        System::assert_has_event(RuntimeEvent::Smith(
            Event::<Runtime>::SmithMembershipRemoved { idty_index: 5 },
        ));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 1,
            issuer: 3,
        }));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 5,
            issuer: 1,
        }));
        System::assert_has_event(RuntimeEvent::Smith(Event::<Runtime>::SmithCertRemoved {
            receiver: 5,
            issuer: 2,
        }));
        assert_eq!(
            Smiths::<Runtime>::get(1),
            Some(SmithMeta {
                status: SmithStatus::Excluded,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![]
            })
        );
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: SmithStatus::Excluded,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![]
            })
        );
        assert_eq!(
            Smiths::<Runtime>::get(5),
            Some(SmithMeta {
                status: SmithStatus::Excluded,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![]
            })
        );
    });
}

#[test]
fn avoid_multiple_events_for_becoming_smith() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![1, 2]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_online(1);
        Pallet::<Runtime>::on_smith_goes_online(2);
        Pallet::<Runtime>::on_smith_goes_online(3);
        // Events cannot be recorded on genesis
        run_to_block(1);
        // State before
        assert_eq!(Smiths::<Runtime>::get(5), None);
        // Try to invite
        assert_ok!(Pallet::<Runtime>::invite_smith(RuntimeOrigin::signed(1), 5));
        assert_ok!(Pallet::<Runtime>::accept_invitation(RuntimeOrigin::signed(
            5
        )));
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(1),
            5
        ));
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(2),
            5
        ));
        System::assert_has_event(RuntimeEvent::Smith(
            Event::<Runtime>::SmithMembershipAdded { idty_index: 5 },
        ));
        run_to_block(2);
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(3),
            5
        ));
        // Should not be promoted again
        assert!(!System::events().iter().any(|record| record.event
            == RuntimeEvent::Smith(Event::<Runtime>::SmithMembershipAdded { idty_index: 5 },)));
    });
}

#[test]
fn should_have_checks_on_certify() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![4]),
            4 => (false, vec![1, 2]),
        ],
    })
    .execute_with(|| {
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_online(1);
        // Initially
        assert_eq!(
            Smiths::<Runtime>::get(1).unwrap(),
            SmithMeta {
                status: Smith,
                expires_on: None,
                issued_certs: vec![4],
                received_certs: vec![2, 3, 4],
            }
        );
        assert_eq!(
            Smiths::<Runtime>::get(2).unwrap(),
            SmithMeta {
                status: Smith,
                expires_on: Some(5),
                issued_certs: vec![1, 4],
                received_certs: vec![3, 4],
            }
        );
        assert_eq!(
            Smiths::<Runtime>::get(3).unwrap(),
            SmithMeta {
                status: Pending,
                expires_on: Some(5),
                issued_certs: vec![1, 2],
                received_certs: vec![4],
            }
        );
        assert_eq!(
            Smiths::<Runtime>::get(4).unwrap(),
            SmithMeta {
                status: Smith,
                expires_on: Some(5),
                issued_certs: vec![1, 2, 3],
                received_certs: vec![1, 2],
            }
        );

        // Tries all possible errors
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(0), 1),
            Error::<Runtime>::OriginHasNeverBeenInvited
        );
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(1), 1),
            Error::<Runtime>::CertificationOfSelfIsForbidden
        );
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(3), 5),
            Error::<Runtime>::CertificationIsASmithPrivilege
        );
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(1), 6),
            Error::<Runtime>::CertificationReceiverMustHaveBeenInvited
        );
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(1), 4),
            Error::<Runtime>::CertificationAlreadyExists
        );

        // #3: state before
        assert_eq!(
            Smiths::<Runtime>::get(3).unwrap(),
            SmithMeta {
                status: Pending,
                expires_on: Some(5),
                issued_certs: vec![1, 2],
                received_certs: vec![4],
            }
        );
        // Try to certify #3
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(1),
            3
        ));
        // #3: state after
        assert_eq!(
            Smiths::<Runtime>::get(3).unwrap(),
            SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(5),
                issued_certs: vec![1, 2],
                received_certs: vec![1, 4],
            }
        );
    });
}

#[test]
fn smith_activity_postpones_expiration() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![]),
            4 => (false, vec![])
        ],
    })
    .execute_with(|| {
        // On session 4 everything is fine
        Pallet::<Runtime>::on_new_session(4);
        assert!(Smiths::<Runtime>::get(1).is_some());
        assert!(Smiths::<Runtime>::get(2).is_some());

        // Smith #2 is online but not #1
        Pallet::<Runtime>::on_smith_goes_online(2);

        // On session 5: exclusion for lack of activity
        Pallet::<Runtime>::on_new_session(5);
        assert_eq!(
            Smiths::<Runtime>::get(1),
            Some(SmithMeta {
                status: SmithStatus::Excluded,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![]
            })
        );
        // issued_certs is empty because #1 was excluded
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![3, 4],
            })
        );

        // Smith #2 goes offline
        Pallet::<Runtime>::on_new_session(6);
        Pallet::<Runtime>::on_smith_goes_offline(2);
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(11),
                issued_certs: vec![],
                received_certs: vec![3, 4],
            })
        );
        // Still not expired on session 10
        Pallet::<Runtime>::on_new_session(10);
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: SmithStatus::Smith,
                expires_on: Some(11),
                issued_certs: vec![],
                received_certs: vec![3, 4],
            })
        );
        // But expired on session 11
        Pallet::<Runtime>::on_new_session(11);
        assert_eq!(
            Smiths::<Runtime>::get(1),
            Some(SmithMeta {
                status: SmithStatus::Excluded,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![]
            })
        );
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: SmithStatus::Excluded,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![]
            })
        );
    });
}

#[test]
fn smith_coming_back_recovers_its_issued_certs() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![1, 4]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // Not activity for Smith #2
        Pallet::<Runtime>::on_smith_goes_online(1);
        Pallet::<Runtime>::on_smith_goes_online(3);
        Pallet::<Runtime>::on_smith_goes_online(4);
        // Smith #2 gets excluded
        Pallet::<Runtime>::on_new_session(5);
        // The issued certs are preserved
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: Excluded,
                expires_on: None,
                issued_certs: vec![1],
                received_certs: vec![]
            })
        );
        // Smith #2 comes back
        assert_ok!(Pallet::<Runtime>::invite_smith(RuntimeOrigin::signed(1), 2));
        assert_ok!(Pallet::<Runtime>::accept_invitation(RuntimeOrigin::signed(
            2
        )));
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(1),
            2
        ));
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(3),
            2
        ));
        // Smith #2 is back with its issued certs recovered, but not its received certs
        assert_eq!(
            Smiths::<Runtime>::get(2),
            Some(SmithMeta {
                status: Smith,
                expires_on: Some(10),
                issued_certs: vec![1],
                received_certs: vec![1, 3]
            })
        );
        Pallet::<Runtime>::on_smith_goes_online(2);
        // We can verify it with the stock rule
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(2),
            3
        ));
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(2),
            4
        ));
        // Max stock is reached (3 = 1 recovered + 2 new)
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(2), 5),
            Error::<Runtime>::CertificationStockFullyConsumed
        );
    });
}

#[test]
fn certifying_on_different_status() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![1, 2]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_online(1);
        Pallet::<Runtime>::on_smith_goes_online(2);
        Pallet::<Runtime>::on_smith_goes_online(3);
        // State before
        assert_eq!(Smiths::<Runtime>::get(5), None);
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(1), 5),
            Error::<Runtime>::CertificationReceiverMustHaveBeenInvited
        );

        // After invitation
        assert_ok!(Pallet::<Runtime>::invite_smith(RuntimeOrigin::signed(1), 5));
        assert_eq!(Smiths::<Runtime>::get(5).unwrap().status, Invited);
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(1), 5),
            Error::<Runtime>::CertificationMustBeAgreed
        );

        // After acceptation
        assert_ok!(Pallet::<Runtime>::accept_invitation(RuntimeOrigin::signed(
            5
        )));
        assert_eq!(Smiths::<Runtime>::get(5).unwrap().status, Pending);
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(1),
            5
        ));
        assert_eq!(Smiths::<Runtime>::get(5).unwrap().status, Pending);
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(2),
            5
        ));
        assert_eq!(Smiths::<Runtime>::get(5).unwrap().status, Smith);

        // After being a smith
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(3),
            5
        ));

        Pallet::<Runtime>::on_smith_goes_online(1);
        Pallet::<Runtime>::on_smith_goes_online(2);
        Pallet::<Runtime>::on_new_session(5);
        assert_eq!(Smiths::<Runtime>::get(1).unwrap().status, Smith);
        assert_eq!(Smiths::<Runtime>::get(2).unwrap().status, Smith);
        assert_eq!(Smiths::<Runtime>::get(5).unwrap().status, Excluded);

        // After being excluded
        assert_err!(
            Pallet::<Runtime>::certify_smith(RuntimeOrigin::signed(1), 5),
            Error::<Runtime>::CertificationOnExcludedIsForbidden
        );
    });
}

#[test]
fn certifying_an_online_smith() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![1, 2]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_online(1);
        Pallet::<Runtime>::on_smith_goes_online(2);
        Pallet::<Runtime>::on_smith_goes_online(3);
        assert_ok!(Pallet::<Runtime>::invite_smith(RuntimeOrigin::signed(1), 5));
        assert_ok!(Pallet::<Runtime>::accept_invitation(RuntimeOrigin::signed(
            5
        )));
        Pallet::<Runtime>::on_new_session(2);
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(1),
            5
        ));
        Pallet::<Runtime>::on_new_session(3);
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(2),
            5
        ));
        // Smith can expire
        assert_eq!(
            Smiths::<Runtime>::get(5),
            Some(SmithMeta {
                status: Smith,
                expires_on: Some(8),
                issued_certs: vec![],
                received_certs: vec![1, 2]
            })
        );
        assert_eq!(ExpiresOn::<Runtime>::get(7), Some(vec![5]));
        assert_eq!(ExpiresOn::<Runtime>::get(8), Some(vec![5]));

        Pallet::<Runtime>::on_smith_goes_online(5);
        // After going online, the expiration disappears
        assert_eq!(
            Smiths::<Runtime>::get(5),
            Some(SmithMeta {
                status: Smith,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![1, 2]
            })
        );
        // ExpiresOn is not unscheduled, but as expires_on has switched to None it's not a problem
        assert_eq!(ExpiresOn::<Runtime>::get(7), Some(vec![5]));
        assert_eq!(ExpiresOn::<Runtime>::get(8), Some(vec![5]));

        // We can receive certification without postponing the expiration (because we are online)
        assert_ok!(Pallet::<Runtime>::certify_smith(
            RuntimeOrigin::signed(3),
            5
        ));
        assert_eq!(
            Smiths::<Runtime>::get(5),
            Some(SmithMeta {
                status: Smith,
                expires_on: None,
                issued_certs: vec![],
                received_certs: vec![1, 2, 3]
            })
        );
    });
}

#[test]
fn invitation_on_non_wot_member() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![1, 2]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // Go online to be able to invite+certify
        Pallet::<Runtime>::on_smith_goes_online(1);
        // State before
        assert_eq!(Smiths::<Runtime>::get(10), None);

        // After invitation
        assert_err!(
            Pallet::<Runtime>::invite_smith(RuntimeOrigin::signed(1), 10),
            Error::<Runtime>::InvitationOfNonMember
        );
    });
}

#[test]
fn losing_wot_membership_cascades_to_smith_members() {
    new_test_ext(GenesisConfig {
        initial_smiths: btreemap![
            1 => (false, vec![2, 3, 4]),
            2 => (false, vec![3, 4]),
            3 => (false, vec![1, 2]),
            4 => (false, vec![]),
        ],
    })
    .execute_with(|| {
        // State before
        assert_eq!(
            Smiths::<Runtime>::get(1),
            Some(SmithMeta {
                status: Smith,
                expires_on: Some(5),
                issued_certs: vec![3],
                received_certs: vec![2, 3, 4],
            })
        );
        assert_eq!(
            Smiths::<Runtime>::get(1).unwrap().issued_certs,
            Vec::<u64>::from([3])
        );
        assert_eq!(
            Smiths::<Runtime>::get(2).unwrap().issued_certs,
            Vec::<u64>::from([1, 3])
        );
        assert_eq!(
            Smiths::<Runtime>::get(3).unwrap().issued_certs,
            Vec::<u64>::from([1, 2])
        );
        assert_eq!(
            Smiths::<Runtime>::get(4).unwrap().issued_certs,
            Vec::<u64>::from([1, 2])
        );

        Pallet::<Runtime>::on_removed_wot_member(1);

        // Excluded
        assert_eq!(
            Smiths::<Runtime>::get(1),
            Some(SmithMeta {
                status: Excluded,
                expires_on: None,
                issued_certs: vec![3],
                received_certs: vec![],
            })
        );
        // Issued certifications updated for certifiers of 1
        assert_eq!(
            Smiths::<Runtime>::get(1).unwrap().issued_certs,
            Vec::<u64>::from([3])
        );
        assert_eq!(
            Smiths::<Runtime>::get(2).unwrap().issued_certs,
            Vec::<u64>::from([3])
        );
        assert_eq!(
            Smiths::<Runtime>::get(3).unwrap().issued_certs,
            Vec::<u64>::from([2])
        );
        assert_eq!(
            Smiths::<Runtime>::get(4).unwrap().issued_certs,
            Vec::<u64>::from([2])
        );
    });
}
