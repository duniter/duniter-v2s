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
use crate::mock::{KIND, Offence, Offences, RuntimeEvent, System, new_test_ext, offence_reports};
use frame_system::{EventRecord, Phase};

#[test]
fn should_report_an_authority_and_trigger_on_offence_and_add_to_blacklist() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![5, 9],
        };

        // when
        Offences::report_offence(vec![], offence).unwrap();

        // then
        assert_eq!(
            offence_reports(KIND, time_slot),
            vec![
                OffenceDetails {
                    offender: 5,
                    reporters: vec![]
                },
                OffenceDetails {
                    offender: 9,
                    reporters: vec![]
                }
            ]
        );
    });
}

#[test]
fn should_not_report_the_same_authority_twice_in_the_same_slot() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![5],
        };
        Offences::report_offence(vec![], offence.clone()).unwrap();

        // when
        // report for the second time
        assert_eq!(
            Offences::report_offence(vec![], offence),
            Err(OffenceError::DuplicateReport)
        );

        // then
        assert_eq!(
            offence_reports(KIND, time_slot),
            vec![OffenceDetails {
                offender: 5,
                reporters: vec![]
            },]
        );
    });
}

#[test]
fn should_report_in_different_time_slot() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let mut offence = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![5],
        };
        Offences::report_offence(vec![], offence.clone()).unwrap();
        System::assert_last_event(
            Event::Offence {
                kind: KIND,
                timeslot: time_slot.encode(),
            }
            .into(),
        );

        // when
        // report for the second time
        offence.time_slot += 1;
        Offences::report_offence(vec![], offence.clone()).unwrap();

        // then
        System::assert_last_event(
            Event::Offence {
                kind: KIND,
                timeslot: offence.time_slot.encode(),
            }
            .into(),
        );
    });
}

#[test]
fn should_deposit_event() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![5],
        };

        // when
        Offences::report_offence(vec![], offence).unwrap();

        // then
        assert_eq!(
            System::events(),
            vec![EventRecord {
                phase: Phase::Initialization,
                event: RuntimeEvent::Offences(crate::Event::Offence {
                    kind: KIND,
                    timeslot: time_slot.encode()
                }),
                topics: vec![],
            }]
        );
    });
}

#[test]
fn doesnt_deposit_event_for_dups() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![5],
        };
        Offences::report_offence(vec![], offence.clone()).unwrap();

        // when
        // report for the second time
        assert_eq!(
            Offences::report_offence(vec![], offence),
            Err(OffenceError::DuplicateReport)
        );

        // then
        // there is only one event.
        assert_eq!(
            System::events(),
            vec![EventRecord {
                phase: Phase::Initialization,
                event: RuntimeEvent::Offences(crate::Event::Offence {
                    kind: KIND,
                    timeslot: time_slot.encode()
                }),
                topics: vec![],
            }]
        );
    });
}

#[test]
fn reports_if_an_offence_is_dup() {
    new_test_ext().execute_with(|| {
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = |time_slot, offenders| Offence {
            validator_set_count: 5,
            time_slot,
            offenders,
        };

        let mut test_offence = offence(time_slot, vec![0]);

        // the report for authority 0 at time slot 42 should not be a known
        // offence
        assert!(
            !<Offences as ReportOffence<_, _, Offence>>::is_known_offence(
                &test_offence.offenders,
                &test_offence.time_slot
            )
        );

        // we report an offence for authority 0 at time slot 42
        Offences::report_offence(vec![], test_offence.clone()).unwrap();

        // the same report should be a known offence now
        assert!(
            <Offences as ReportOffence<_, _, Offence>>::is_known_offence(
                &test_offence.offenders,
                &test_offence.time_slot
            )
        );

        // and reporting it again should yield a duplicate report error
        assert_eq!(
            Offences::report_offence(vec![], test_offence.clone()),
            Err(OffenceError::DuplicateReport)
        );

        // after adding a new offender to the offence report
        test_offence.offenders.push(1);

        // it should not be a known offence anymore
        assert!(
            !<Offences as ReportOffence<_, _, Offence>>::is_known_offence(
                &test_offence.offenders,
                &test_offence.time_slot
            )
        );

        // and reporting it again should work without any error
        assert_eq!(
            Offences::report_offence(vec![], test_offence.clone()),
            Ok(())
        );

        // creating a new offence for the same authorities on the next slot
        // should be considered a new offence and thefore not known
        let test_offence_next_slot = offence(time_slot + 1, vec![0, 1]);
        assert!(
            !<Offences as ReportOffence<_, _, Offence>>::is_known_offence(
                &test_offence_next_slot.offenders,
                &test_offence_next_slot.time_slot
            )
        );
    });
}

#[test]
fn should_properly_count_offences() {
    // We report two different authorities for the same issue. Ultimately, the 1st authority
    // should have `count` equal 2 and the count of the 2nd one should be equal to 1.
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence1 = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![5],
        };
        let offence2 = Offence {
            validator_set_count: 5,
            time_slot,
            offenders: vec![4],
        };
        Offences::report_offence(vec![], offence1).unwrap();

        // when
        // report for the second time
        Offences::report_offence(vec![], offence2).unwrap();

        // then
        // the 1st authority should have count 2 and the 2nd one should be reported only once.
        assert_eq!(
            offence_reports(KIND, time_slot),
            vec![
                OffenceDetails {
                    offender: 5,
                    reporters: vec![]
                },
                OffenceDetails {
                    offender: 4,
                    reporters: vec![]
                },
            ]
        );
    });
}
