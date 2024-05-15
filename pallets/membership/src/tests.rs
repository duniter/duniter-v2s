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

use crate::{mock::*, Error, Event, MembershipRemovalReason};
use frame_support::{assert_noop, assert_ok};
use maplit::btreemap;
use sp_membership::{traits::*, MembershipData};

fn default_gen_conf() -> MembershipConfig {
    MembershipConfig {
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
            Membership::membership(0),
            Some(MembershipData { expire_on: 3 })
        );
        assert_eq!(Membership::members_count(), 1);
    });
}

/// test membership expiration
// membership should expire
#[test]
fn test_membership_expiration() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        // Membership 0 should not expired on block #2
        run_to_block(2);
        assert!(Membership::is_member(&0));
        // Membership 0 should expire on block #3
        run_to_block(3);
        assert!(!Membership::is_member(&0));
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Expired,
        }));
    });
}

/// test membership renewal (triggered automatically after distance evaluation)
#[test]
fn test_membership_renewal() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        // membership still valid at block 2
        run_to_block(2);
        assert!(Membership::is_member(&0));
        // Membership 0 can be renewed
        assert_ok!(Membership::try_renew_membership(0));
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipRenewed {
            member: 0,
            expire_on: 2 + <Test as crate::Config>::MembershipPeriod::get(),
        }));
        // membership should not expire at block 3 to 6 because it has been renewed
        run_to_block(3);
        assert!(Membership::is_member(&0));
        run_to_block(6);
        assert!(Membership::is_member(&0));
        // membership should expire at block 7 (2+5)
        run_to_block(7);
        assert!(!Membership::is_member(&0));
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipRemoved {
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
        assert!(!Membership::is_member(&1));
        // Membership 1 can not be renewed
        assert_noop!(
            Membership::try_renew_membership(1),
            Error::<Test>::MembershipNotFound,
        );
        run_to_block(3);
        assert!(!Membership::is_member(&1));
    });
}

/// test membership revocation
#[test]
fn test_membership_revocation() {
    new_test_ext(default_gen_conf()).execute_with(|| {
        run_to_block(1);
        // Membership 0 can be revocable on block #1
        Membership::do_remove_membership(0, MembershipRemovalReason::Revoked);
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Revoked,
        }));
        assert_eq!(Membership::membership(0), None);

        // Membership 0 can re-claim membership
        run_to_block(5);
        assert_ok!(Membership::try_add_membership(0));
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipAdded {
            member: 0,
            expire_on: 5 + <Test as crate::Config>::MembershipPeriod::get(),
        }));
    });
}

/// test membership workflow
// - claim membership
// - renew membership
// - membership expiry
#[test]
fn test_membership_workflow() {
    new_test_ext(Default::default()).execute_with(|| {
        // - Then, idty 0 claim membership
        run_to_block(2);
        assert_ok!(Membership::try_add_membership(0));
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipAdded {
            member: 0,
            expire_on: 2 + <Test as crate::Config>::MembershipPeriod::get(),
        }));

        // - Then, idty 0 claim renewal, should success
        run_to_block(2);
        assert_ok!(Membership::try_renew_membership(0));

        // idty 0 should still be member until membership period ended
        run_to_block(6); // 2 + 5 - 1
        assert!(Membership::is_member(&0));

        // - Then, idty 0 should expire after membership period
        run_to_block(7); // 2 + 5
        assert!(!Membership::is_member(&0));
        System::assert_has_event(RuntimeEvent::Membership(Event::MembershipRemoved {
            member: 0,
            reason: MembershipRemovalReason::Expired,
        }));
    });
}
