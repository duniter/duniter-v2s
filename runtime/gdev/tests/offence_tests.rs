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
use frame_support::{
    assert_ok,
    traits::{ValidatorSet, ValidatorSetWithIdentification},
};
use gdev_runtime::*;
use pallet_im_online as im_online;
use pallet_im_online::UnresponsivenessOffence;
use pallet_session::historical::IdentificationTuple;
use sp_runtime::traits::Convert;
use sp_staking::offence::ReportOffence;

#[test]
fn test_imonline_offence() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        let session_index = Session::current_index();
        let current_validators = <Runtime as im_online::Config>::ValidatorSet::validators();
        // Construct an offence where all validators (member: 1) are offenders
        let offenders = current_validators
            .into_iter()
            .filter_map(|id| {
                <<Runtime as im_online::Config>::ValidatorSet as ValidatorSetWithIdentification<
                    sp_runtime::AccountId32,
                >>::IdentificationOf::convert(id.clone())
                .map(|full_id| (id, full_id))
            })
            .collect::<Vec<IdentificationTuple<Runtime>>>();
        let keys = pallet_im_online::Keys::<Runtime>::get();
        let validator_set_count = keys.len() as u32;
        let offence = UnresponsivenessOffence {
            session_index,
            validator_set_count,
            offenders,
        };
        assert_ok!(
            <Runtime as pallet_im_online::Config>::ReportUnresponsiveness::report_offence(
                vec![],
                offence
            )
        );
        // An offence is deposited
        System::assert_has_event(RuntimeEvent::Offences(pallet_offences::Event::Offence {
            kind: *b"im-online:offlin",
            timeslot: vec![0, 0, 0, 0],
        }));
        // Offenders are punished
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberGoOffline { member: 1 },
        ));
        assert_eq!(AuthorityMembers::blacklist().len(), 0);
    })
}
#[test]
fn test_grandpa_offence() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        let session_index = Session::current_index();
        let current_validators = <Runtime as pallet_im_online::Config>::ValidatorSet::validators();
        // Construct an offence where all validators (member: 1) are offenders
        let mut offenders = current_validators
            .into_iter()
            .filter_map(|id| {
                <Runtime as pallet_session::historical::Config>::FullIdentificationOf::convert(
                    id.clone(),
                )
                .map(|full_id| (id, full_id))
            })
            .collect::<Vec<IdentificationTuple<Runtime>>>();
        let keys = pallet_im_online::Keys::<Runtime>::get();
        let validator_set_count = keys.len() as u32;
        let time_slot = pallet_grandpa::TimeSlot {
            set_id: 0,
            round: 0,
        };
        let offence = pallet_grandpa::EquivocationOffence {
            time_slot,
            session_index,
            validator_set_count,
            offender: offenders.pop().unwrap(),
        };
        assert_ok!(Offences::report_offence(vec![], offence));
        // An offence is deposited
        System::assert_has_event(RuntimeEvent::Offences(pallet_offences::Event::Offence {
            kind: *b"grandpa:equivoca",
            timeslot: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }));
        // Offenders are punished
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberGoOffline { member: 1 },
        ));
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberAddedToBlacklist { member: 1 },
        ));
        assert_eq!(AuthorityMembers::blacklist().len(), 1);
    })
}
#[test]
fn test_babe_offence() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        let session_index = Session::current_index();
        let current_validators = <Runtime as pallet_im_online::Config>::ValidatorSet::validators();
        // Construct an offence where all validators (member: 1) are offenders
        let mut offenders = current_validators
            .into_iter()
            .filter_map(|id| {
                <Runtime as pallet_session::historical::Config>::FullIdentificationOf::convert(
                    id.clone(),
                )
                .map(|full_id| (id, full_id))
            })
            .collect::<Vec<IdentificationTuple<Runtime>>>();
        let keys = pallet_im_online::Keys::<Runtime>::get();
        let validator_set_count = keys.len() as u32;
        let offence = pallet_babe::EquivocationOffence {
            slot: 0u64.into(),
            session_index,
            validator_set_count,
            offender: offenders.pop().unwrap(),
        };
        assert_ok!(Offences::report_offence(vec![], offence));
        // An offence is deposited
        System::assert_has_event(RuntimeEvent::Offences(pallet_offences::Event::Offence {
            kind: *b"babe:equivocatio",
            timeslot: vec![0, 0, 0, 0, 0, 0, 0, 0],
        }));
        // Offenders are punished
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberGoOffline { member: 1 },
        ));
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberAddedToBlacklist { member: 1 },
        ));
        assert_eq!(AuthorityMembers::blacklist().len(), 1);
    })
}
