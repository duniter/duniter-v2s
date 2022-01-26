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

use super::*;
use crate::mock::*;
use crate::MemberData;
use frame_support::{assert_err, assert_ok};
use sp_runtime::testing::UintAuthorityId;

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
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 5,
            })
        );
        assert_eq!(
            AuthorityMembers::member(6),
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 5,
            })
        );
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 5,
            })
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

#[test]
fn test_max_keys_life_rule() {
    new_test_ext(3).execute_with(|| {
        run_to_block(11);

        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![3, 6, 9]);

        // Member 3 and 6 rotate their sessions keys
        assert_ok!(AuthorityMembers::set_session_keys(
            Origin::signed(3),
            3,
            UintAuthorityId(3).into()
        ),);
        assert_ok!(AuthorityMembers::set_session_keys(
            Origin::signed(6),
            6,
            UintAuthorityId(6).into()
        ),);

        // Verify state
        assert_eq!(
            AuthorityMembers::member(3),
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 7,
            })
        );
        assert_eq!(
            AuthorityMembers::member(6),
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 7,
            })
        );

        // Member 9 should expire at session 5
        run_to_block(26);
        assert_eq!(Session::current_index(), 5);
        assert_eq!(AuthorityMembers::online(), vec![3, 6]);
        assert_eq!(AuthorityMembers::member(9), None);

        // Member 9 should be "deprogrammed" but still in the authorities set for 1 session
        assert_eq!(Session::queued_keys().len(), 2);
        assert_eq!(Session::queued_keys()[0].0, 3);
        assert_eq!(Session::queued_keys()[1].0, 6);
        assert_eq!(Session::validators(), vec![3, 6, 9]);

        // Member 9 should be **effectively** out at session 6
        run_to_block(31);
        assert_eq!(Session::current_index(), 6);
        assert_eq!(Session::validators(), vec![3, 6]);
    });
}

#[test]
fn test_go_offline() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 9 should be able to go offline
        assert_ok!(AuthorityMembers::go_offline(Origin::signed(9), 9),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), vec![9]);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 5,
            })
        );

        // Member 9 should be "deprogrammed" at the next session
        run_to_block(5);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData {
                expire_on_session: 4,
                must_rotate_keys_before: 5,
            })
        );
        assert_eq!(AuthorityMembers::members_expire_on(4), vec![9],);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![3, 6, 9]);
        assert_eq!(Session::queued_keys().len(), 2);
        assert_eq!(Session::queued_keys()[0].0, 3);
        assert_eq!(Session::queued_keys()[1].0, 6);

        // Member 9 should be **effectively** out at session 2
        run_to_block(10);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![3, 6]);

        // Member 9 should be removed at session 4
        run_to_block(20);
        assert_eq!(Session::current_index(), 4);
        assert_eq!(Session::validators(), vec![3, 6]);
        assert_eq!(AuthorityMembers::members_expire_on(4), EMPTY);
        assert_eq!(AuthorityMembers::member(9), None);
    });
}

#[test]
fn test_go_online() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 12 should be able to set his session keys
        assert_ok!(AuthorityMembers::set_session_keys(
            Origin::signed(12),
            12,
            UintAuthorityId(12).into(),
        ));
        assert_eq!(
            AuthorityMembers::member(12),
            Some(MemberData {
                expire_on_session: 2,
                must_rotate_keys_before: 5,
            })
        );

        // Member 12 should be able to go online
        assert_ok!(AuthorityMembers::go_online(Origin::signed(12), 12),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), vec![12]);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(12),
            Some(MemberData {
                expire_on_session: 2,
                must_rotate_keys_before: 5,
            })
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

        // Member 12 set his session keys then go online
        assert_ok!(AuthorityMembers::set_session_keys(
            Origin::signed(12),
            12,
            UintAuthorityId(12).into(),
        ));
        assert_eq!(AuthorityMembers::authorities_counter(), 3);
        assert_ok!(AuthorityMembers::go_online(Origin::signed(12), 12),);

        // Member 15 can't go online because there is already 4 authorities "planned"
        assert_ok!(AuthorityMembers::set_session_keys(
            Origin::signed(15),
            15,
            UintAuthorityId(15).into(),
        ));
        assert_eq!(AuthorityMembers::authorities_counter(), 4);
        assert_err!(
            AuthorityMembers::go_online(Origin::signed(15), 15),
            Error::<Test>::TooManyAuthorities,
        );

        // If member 3 go_offline, member 15 can go_online
        assert_ok!(AuthorityMembers::go_offline(Origin::signed(3), 3),);
        assert_eq!(AuthorityMembers::authorities_counter(), 3);
        assert_ok!(AuthorityMembers::go_online(Origin::signed(15), 15),);
        assert_eq!(AuthorityMembers::authorities_counter(), 4);
    });
}

#[test]
fn test_go_online_then_go_offline_in_same_session() {
    new_test_ext(3).execute_with(|| {
        run_to_block(1);

        // Member 12 set his session keys & go online
        assert_ok!(AuthorityMembers::set_session_keys(
            Origin::signed(12),
            12,
            UintAuthorityId(12).into(),
        ));
        assert_ok!(AuthorityMembers::go_online(Origin::signed(12), 12),);

        run_to_block(2);

        // Member 12 should be able to go offline at the same session to "cancel" his previous
        // action
        assert_ok!(AuthorityMembers::go_offline(Origin::signed(12), 12),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(12),
            Some(MemberData {
                expire_on_session: 2,
                must_rotate_keys_before: 5,
            })
        );
    });
}

#[test]
fn test_go_offline_then_go_online_in_same_session() {
    new_test_ext(3).execute_with(|| {
        run_to_block(6);

        // Member 9 go offline
        assert_ok!(AuthorityMembers::go_offline(Origin::signed(9), 9),);

        run_to_block(7);

        // Member 9 should be able to go online at the same session to "cancel" his previous action
        assert_ok!(AuthorityMembers::go_online(Origin::signed(9), 9),);

        // Verify state
        assert_eq!(AuthorityMembers::incoming(), EMPTY);
        assert_eq!(AuthorityMembers::online(), vec![3, 6, 9]);
        assert_eq!(AuthorityMembers::outgoing(), EMPTY);
        assert_eq!(
            AuthorityMembers::member(9),
            Some(MemberData {
                expire_on_session: 0,
                must_rotate_keys_before: 5,
            })
        );
    });
}
