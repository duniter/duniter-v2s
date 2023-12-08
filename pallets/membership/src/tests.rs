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
use crate::MembershipRemovalReason;
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};
use maplit::btreemap;
use sp_membership::traits::*;
use sp_membership::MembershipData;
use sp_runtime::traits::IsMember;

// alias
type RtEvent = RuntimeEvent;

fn default_gen_conf() -> DefaultMembershipConfig {
    DefaultMembershipConfig {
        memberships: btreemap![
            0 => MembershipData {
                expire_on: 3,
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
            Some(MembershipData { expire_on: 3 })
        );
        assert_eq!(DefaultMembership::members_count(), 1);
    });
}

#[test]
fn test_membership_already_acquired() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Membership 0 cannot be reclaimed because there is no membership request
        assert_noop!(
            DefaultMembership::claim_membership(RuntimeOrigin::signed(0)),
            Error::<Test, _>::MembershipRequestNotFound
        );
    });
}

#[test]
fn test_membership_request_not_found() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Membership 0 cannot be reclaimed
        assert_noop!(
            DefaultMembership::claim_membership(RuntimeOrigin::signed(1)),
            Error::<Test, _>::MembershipRequestNotFound
        );
    });
}

/// test membership expiration
// membership should be moved to pending membership and expire after
#[test]
fn test_membership_expiration() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        // Membership 0 should not expired on block #2
        run_to_block(2);
        assert!(DefaultMembership::is_member(&0));
        // Membership 0 should expire on block #3
        run_to_block(3);
        assert!(!DefaultMembership::is_member(&0));
        System::assert_has_event(RtEvent::DefaultMembership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Expired,
        }));
        // it should be added to pending membership and expire on block #6
        run_to_block(6);
        System::assert_has_event(RtEvent::DefaultMembership(
            Event::PendingMembershipExpired { member: 0 },
        ));
    });
}

/// test membership renewal
// there is no limit for membership renewal outside wot rules (number of certs, distance rule)
#[test]
fn test_membership_renewal() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        // membership still valid at block 2
        run_to_block(2);
        assert!(DefaultMembership::is_member(&0));
        // Membership 0 can be renewed
        assert_ok!(DefaultMembership::renew_membership(RuntimeOrigin::signed(
            0
        ),));
        System::assert_has_event(RtEvent::DefaultMembership(Event::MembershipAdded {
            member: 0,
            expire_on: 2 + <Test as crate::Config>::MembershipPeriod::get(),
        }));
        // membership should not expire at block 3 to 6 because it has been renewed
        run_to_block(3);
        assert!(DefaultMembership::is_member(&0));
        run_to_block(6);
        assert!(DefaultMembership::is_member(&0));
        // membership should expire at block 7 (2+5)
        run_to_block(7);
        assert!(!DefaultMembership::is_member(&0));
        System::assert_has_event(RtEvent::DefaultMembership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Expired,
        }));
    });
}

/// test membership renewal for non member identity
#[test]
fn test_membership_renewal_nope() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(2);
        assert!(!DefaultMembership::is_member(&1));
        // Membership 1 can not be renewed
        assert_noop!(
            DefaultMembership::renew_membership(RuntimeOrigin::signed(1)),
            Error::<Test, _>::MembershipNotFound,
        );
        run_to_block(3);
        assert!(!DefaultMembership::is_member(&1));
    });
}

/// test membership revocation
#[test]
fn test_membership_revocation() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Membership 0 can be revocable on block #1
        assert_ok!(DefaultMembership::revoke_membership(RuntimeOrigin::signed(
            0
        ),));
        System::assert_has_event(RtEvent::DefaultMembership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Revoked,
        }));

        // Membership 0 can re-request membership
        run_to_block(5);
        assert_ok!(DefaultMembership::request_membership(
            RuntimeOrigin::signed(0),
        ));
        System::assert_has_event(RtEvent::DefaultMembership(Event::PendingMembershipAdded {
            member: 0,
            expire_on: 5 + <Test as crate::Config>::PendingMembershipPeriod::get(),
        }));
    });
}

/// test pending membership expiration
#[test]
fn test_pending_membership_expiration() {
    new_test_ext(Default::default()).execute_with(|| {
        // Idty 0 request membership
        run_to_block(1);
        assert_ok!(DefaultMembership::request_membership(
            RuntimeOrigin::signed(0),
        ));
        System::assert_has_event(RtEvent::DefaultMembership(Event::PendingMembershipAdded {
            member: 0,
            expire_on: 1 + <Test as crate::Config>::PendingMembershipPeriod::get(),
        }));

        // Then, idty 0 shold still in pending memberships until PendingMembershipPeriod ended
        run_to_block(PendingMembershipPeriod::get());
        assert!(DefaultMembership::is_in_pending_memberships(0));

        // Then, idty 0 request should expire after PendingMembershipPeriod
        run_to_block(1 + PendingMembershipPeriod::get());
        assert!(!DefaultMembership::is_in_pending_memberships(0));
        System::assert_has_event(RtEvent::DefaultMembership(
            Event::PendingMembershipExpired { member: 0 },
        ));
    })
}

/// test membership workflow
// - request membership
// - claim membership
// - renew membership
// - membership expiry
#[test]
fn test_membership_workflow() {
    new_test_ext(Default::default()).execute_with(|| {
        // - Idty 0 request membership
        run_to_block(1);
        assert_ok!(DefaultMembership::request_membership(
            RuntimeOrigin::signed(0),
        ));
        System::assert_has_event(RtEvent::DefaultMembership(Event::PendingMembershipAdded {
            member: 0,
            expire_on: 1 + <Test as crate::Config>::PendingMembershipPeriod::get(),
        }));

        // - Then, idty 0 claim membership
        run_to_block(2);
        assert_ok!(DefaultMembership::claim_membership(RuntimeOrigin::signed(
            0
        ),));
        System::assert_has_event(RtEvent::DefaultMembership(Event::MembershipAdded {
            member: 0,
            expire_on: 2 + <Test as crate::Config>::MembershipPeriod::get(),
        }));

        // - Then, idty 0 claim renewal, should success
        run_to_block(2);
        assert_ok!(DefaultMembership::renew_membership(RuntimeOrigin::signed(
            0
        ),));

        // idty 0 should still be member until membership period ended
        run_to_block(6); // 2 + 5 - 1
        assert!(DefaultMembership::is_member(&0));

        // - Then, idty 0 should expire after membership period
        run_to_block(7); // 2 + 5
        assert!(!DefaultMembership::is_member(&0));
        System::assert_has_event(RtEvent::DefaultMembership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Expired,
        }));
    });
}
