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

use crate::mock::Event as RuntimeEvent;
use crate::mock::*;
use crate::{Error, Event};
use frame_support::assert_ok;
use maplit::btreemap;
use sp_membership::traits::{IsInPendingMemberships, IsMember};
use sp_membership::MembershipData;

fn default_gen_conf() -> DefaultMembershipConfig {
    DefaultMembershipConfig {
        memberships: btreemap![
            0 => MembershipData {
                expire_on: 3,
                renewable_on: 2
            }
        ],
    }
}

#[test]
fn test_genesis_build() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Verify state
        assert_eq!(
            DefaultMembership::membership(0),
            Some(MembershipData {
                expire_on: 3,
                renewable_on: 2
            })
        );
    });
}

#[test]
fn test_membership_not_yet_renewable() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Merbership 0 cannot be renewed before #2
        assert_eq!(
            DefaultMembership::renew_membership(Origin::signed(0), 0),
            Err(Error::<Test, _>::MembershipNotYetRenewable.into())
        );
    });
}

#[test]
fn test_membership_request_not_found() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Merbership 0 cannot be reclaimed
        assert_eq!(
            DefaultMembership::claim_membership(Origin::signed(0), 0),
            Err(Error::<Test, _>::MembershipRequestNotFound.into())
        );
    });
}

#[test]
fn test_membership_renewal() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(2);
        // Merbership 0 can be renewable on block #2
        assert_ok!(DefaultMembership::renew_membership(Origin::signed(0), 0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipRenewed(0))
        );
    });
}

#[test]
fn test_membership_expiration() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        // Merbership 0 should not expired on block #2
        run_to_block(2);
        assert!(DefaultMembership::is_member(&0),);
        // Merbership 0 should expire on block #3
        run_to_block(3);
        assert!(!DefaultMembership::is_member(&0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipExpired(0))
        );
    });
}

#[test]
fn test_membership_revocation() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Merbership 0 can be revocable on block #1
        assert_ok!(DefaultMembership::revoke_membership(Origin::signed(0), 0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipRevoked(0))
        );

        // Membership 0 can't request membership before the end of RevokePeriod (1 + 4 = 5)
        run_to_block(2);
        assert_eq!(
            DefaultMembership::request_membership(Origin::signed(0), 0),
            Err(Error::<Test, _>::MembershipRevokedRecently.into())
        );

        // Membership 0 can request membership after the end of RevokePeriod (1 + 4 = 5)
        run_to_block(5);
        assert_ok!(DefaultMembership::request_membership(Origin::signed(0), 0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipRequested(0))
        );
    });
}

#[test]
fn test_pending_membership_expiration() {
    new_test_ext(Default::default()).execute_with(|| {
        // Idty 0 request membership
        run_to_block(1);
        assert_ok!(DefaultMembership::request_membership(Origin::signed(0), 0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipRequested(0))
        );

        // Then, idty 0 shold still in pending memberships until PendingMembershipPeriod ended
        run_to_block(PendingMembershipPeriod::get());
        assert!(DefaultMembership::is_in_pending_memberships(0),);

        // Then, idty 0 request should expire after PendingMembershipPeriod
        run_to_block(1 + PendingMembershipPeriod::get());
        assert!(!DefaultMembership::is_in_pending_memberships(0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::PendingMembershipExpired(0))
        );
    })
}

#[test]
fn test_membership_workflow() {
    new_test_ext(Default::default()).execute_with(|| {
        // Idty 0 request membership
        run_to_block(1);
        assert_ok!(DefaultMembership::request_membership(Origin::signed(0), 0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipRequested(0))
        );

        // Then, idty 0 claim membership
        run_to_block(2);
        assert_ok!(DefaultMembership::claim_membership(Origin::signed(0), 0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipAcquired(0))
        );

        // Then, idty 0 claim renewal, should fail
        run_to_block(3);
        assert_eq!(
            DefaultMembership::renew_membership(Origin::signed(0), 0),
            Err(Error::<Test, _>::MembershipNotYetRenewable.into())
        );

        // Then, idty 0 claim renewal after renewable period, should success
        run_to_block(2 + RenewablePeriod::get());
        assert_ok!(DefaultMembership::renew_membership(Origin::signed(0), 0),);

        // Then, idty 0 shoul still member until membership period ended
        run_to_block(2 + RenewablePeriod::get() + MembershipPeriod::get() - 1);
        assert!(DefaultMembership::is_member(&0));

        // Then, idty 0 shoul expire after membership period
        run_to_block(2 + RenewablePeriod::get() + MembershipPeriod::get());
        assert!(!DefaultMembership::is_member(&0),);
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultMembership(Event::MembershipExpired(0))
        );
    });
}
