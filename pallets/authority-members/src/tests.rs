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

use super::*;
use crate::mock::*;
use crate::MemberData;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::RawOrigin;
use sp_runtime::testing::UintAuthorityId;
use sp_runtime::traits::BadOrigin;
use sp_staking::offence::OffenceDetails;

const EMPTY: Vec<u64> = Vec::new();

#[test]
fn test_genesis_build() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);
        // Verify AuthorityMembers state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(3),
            Some(MemberData { owner_key: 3 })
        );
        assert_eq!(
            AuthorityMembers::member(6),
            Some(MemberData { owner_key: 6 })
        );
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData { owner_key: 9 })
        );

        // Verify Session state
        assert_eq!(Session::current_index(), 0);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
    });
}

#[test]
fn test_new_session_shoud_not_change_authorities_set() {
    new_test_ext(3).execute_with(|| {
        run_to_block(6);

        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
    });
}

/// tests consequences of go_offline call
#[test]
fn test_go_offline() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 9 should be able to go offline
        assert_ok!(AuthorityMembers::go_offline(RuntimeOrigin::signed(9)),);

        // Verify state
        assert_eq!(Session::current_index(), 0); // we are currently at session 0
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), vec![9]);
        assert_eq!(AuthorityMembers::blacklist(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData { owner_key: 9 })
        );

        // Member 9 should be outgoing at the next session (session 1).
        // They should be out at session 2.
        run_to_block(5);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData { owner_key: 9 })
        );
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
        assert_eq!(Session::queued_keys().len(), 2);
        assert_eq!(Session::queued_keys()[0].0, 3);
        assert_eq!(Session::queued_keys()[1].0, 6);

        run_to_block(10);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![3, 6]);
    });
}

/// tests consequences of go_online call
#[test]
fn test_go_online() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 12 should be able to set their session keys
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(12),
            UintAuthorityId(12).into(),
        ));
        assert_eq!(
            AuthorityMembers::member(12),
            Some(MemberData { owner_key: 12 })
        );

        // Member 12 should be able to go online
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(12)),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), vec![12]);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(12),
            Some(MemberData { owner_key: 12 })
        );

        // Member 12 should be "programmed" at the next session
        run_to_block(5);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
        assert_eq!(Session::queued_keys().len(), 4);
        assert_eq!(Session::queued_keys()[0].0, 3);
        assert_eq!(Session::queued_keys()[1].0, 6);
        assert_eq!(Session::queued_keys()[2].0, 9);
        assert_eq!(Session::queued_keys()[3].0, 12);

        // Member 12 should be **effectively** in the authorities set in 2 sessions
        run_to_block(10);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![3, 6, 9, 12]);
    });
}

#[test]
fn test_too_many_authorities() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 12 sets their session keys then go online
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(12),
            UintAuthorityId(12).into(),
        ));
        assert_eq!(AuthorityMembers::authorities_counter(), 3);
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(12)),);

        // Member 15 can't go online because there is already 4 authorities "planned"
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(15),
            UintAuthorityId(15).into(),
        ));
        assert_eq!(AuthorityMembers::authorities_counter(), 4);
        assert_noop!(
            AuthorityMembers::go_online(RuntimeOrigin::signed(15)),
            Error::<Test>::TooManyAuthorities,
        );

        // If member 3 go_offline, member 15 can go_online
        assert_ok!(AuthorityMembers::go_offline(RuntimeOrigin::signed(3)),);
        assert_eq!(AuthorityMembers::authorities_counter(), 3);
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(15)),);
        assert_eq!(AuthorityMembers::authorities_counter(), 4);
        assert_ok!(AuthorityMembers::remove_member(RawOrigin::Root.into(), 15));
        assert_eq!(AuthorityMembers::authorities_counter(), 3);
    });
}

#[test]
fn test_go_online_then_go_offline_in_same_session() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 12 sets their session keys & go online
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(12),
            UintAuthorityId(12).into(),
        ));
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(12)),);

        run_to_block(2);

        // Member 12 should be able to go offline at the same session to "cancel" their previous
        // action
        assert_ok!(AuthorityMembers::go_offline(RuntimeOrigin::signed(12)),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(12),
            Some(MemberData { owner_key: 12 })
        );
    });
}

#[test]
fn test_go_offline_then_go_online_in_same_session() {
    new_test_ext(3).execute_with(|| {
        run_to_block(6);

        // Member 9 go offline
        assert_ok!(AuthorityMembers::go_offline(RuntimeOrigin::signed(9)),);

        run_to_block(7);

        // Member 9 should be able to go online at the same session to "cancel" their previous action
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(9)),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData { owner_key: 9 })
        );
    });
}

// === offence handling tests below ===

/// test offence handling with disconnect strategy
// the offenders should be disconnected (same as go_offline)
// they should be able to go_online after
#[test]
fn test_offence_disconnect() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        on_offence(
            &[OffenceDetails {
                offender: (9, ()),
                reporters: vec![],
            }],
            pallet_offences::SlashStrategy::Disconnect,
        );
        on_offence(
            &[OffenceDetails {
                offender: (3, ()),
                reporters: vec![],
            }],
            pallet_offences::SlashStrategy::Disconnect,
        );

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), vec![3, 9]);
        assert_eq!(AuthorityMembers::blacklist(), EMPTY);

        // Member 9 and 3 should be outgoing at the next session
        run_to_block(5);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData { owner_key: 9 })
        );
        assert_eq!(
            AuthorityMembers::member(3),
            Some(MemberData { owner_key: 3 })
        );
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
        assert_eq!(Session::queued_keys().len(), 1);
        assert_eq!(Session::queued_keys()[0].0, 6);

        // Member 9 and 3 should be out at session 2
        run_to_block(10);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![6]);

        // Member 9 and 3 should be allowed to set session keys and go online
        run_to_block(25);
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(9),
            UintAuthorityId(9).into(),
        ));
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(3),
            UintAuthorityId(3).into(),
        ));
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(9)),);
        assert_ok!(AuthorityMembers::go_online(RuntimeOrigin::signed(3)),);

        // Report an offence again
        run_to_block(35);
        on_offence(
            &[OffenceDetails {
                offender: (3, ()),
                reporters: vec![],
            }],
            pallet_offences::SlashStrategy::Disconnect,
        );

        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), vec![3]);
        assert_eq!(AuthorityMembers::blacklist(), EMPTY);
    });
}

/// test offence handling with blacklist strategy
// member 9 is offender, should be blacklisted
#[test]
fn test_offence_black_list() {
    new_test_ext(3).execute_with(|| {
        // at block 0 begins session 0
        run_to_block(1);

        on_offence(
            &[OffenceDetails {
                offender: (9, ()),
                reporters: vec![],
            }],
            pallet_offences::SlashStrategy::Blacklist,
        );

        // Verify state
        // same as `test_go_offline`
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), vec![9]);
        assert_eq!(AuthorityMembers::blacklist(), vec![9]);

        // Member 9 should be outgoing at the next session
        run_to_block(5);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::blacklist(), vec![9]); // still in blacklist

        // Member 9 should be out at session 2
        run_to_block(10);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![3, 6]);
        assert_eq!(AuthorityMembers::blacklist(), vec![9]); // still in blacklist
    });
}

/// tests that blacklisting prevents 9 from going online
#[test]
fn test_offence_black_list_prevent_from_going_online() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        on_offence(
            &[OffenceDetails {
                offender: (9, ()),
                reporters: vec![],
            }],
            pallet_offences::SlashStrategy::Blacklist,
        );

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), vec![9]);
        assert_eq!(AuthorityMembers::blacklist(), vec![9]);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData { owner_key: 9 })
        );

        // for detail, see `test_go_offline`
        // Member 9 is "deprogrammed" at the next session
        // Member 9 is **effectively** out at session 2
        // Member 9 is removed at session 4

        // Member 9 should not be allowed to go online
        run_to_block(25);
        assert_ok!(AuthorityMembers::set_session_keys(
            RuntimeOrigin::signed(9),
            UintAuthorityId(9).into(),
        ));
        assert_err!(
            AuthorityMembers::go_online(RuntimeOrigin::signed(9)),
            Error::<Test>::MemberBlacklisted
        );

        // Should not be able to auto remove from blacklist
        assert_err!(
            AuthorityMembers::remove_member_from_blacklist(RuntimeOrigin::signed(9), 9),
            BadOrigin
        );
        assert_eq!(AuthorityMembers::blacklist(), vec![9]);

        // Authorized should be able to remove from blacklist
        assert_ok!(AuthorityMembers::remove_member_from_blacklist(
            RawOrigin::Root.into(),
            9
        ));
        assert_eq!(AuthorityMembers::blacklist(), EMPTY);
        System::assert_last_event(Event::MemberRemovedFromBlacklist { member: 9 }.into());

        // Authorized should not be able to remove a non-existing member from blacklist
        assert_err!(
            AuthorityMembers::remove_member_from_blacklist(RawOrigin::Root.into(), 9),
            Error::<Test>::MemberNotBlacklisted
        );
    });
}
