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
//use frame_system::{EventRecord, Phase};
use maplit::btreemap;
use sp_std::collections::btree_map::BTreeMap;

#[test]
fn test_must_receive_cert_before_can_issue() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: BTreeMap::new(),
    })
    .execute_with(|| {
        assert_eq!(
            DefaultCertification::add_cert(Origin::signed(0), 0, 1),
            Err(Error::<Test, _>::NotEnoughCertReceived.into())
        );
    });
}

#[test]
fn test_cannot_certify_self() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => 5,
                2 => 5,
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(2);

        assert_eq!(
            DefaultCertification::add_cert(Origin::signed(0), 0, 0),
            Err(Error::<Test, _>::CannotCertifySelf.into())
        );
    });
}

#[test]
fn test_genesis_build() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => 7,
                2 => 9,
            ],
            1 => btreemap![
                0 => 10,
                2 => 3,
            ],
            2 => btreemap![
                0 => 5,
                1 => 4,
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(1);
        // Verify state of idty 0
        assert_eq!(
            DefaultCertification::idty_cert_meta(0),
            crate::IdtyCertMeta {
                issued_count: 2,
                next_issuable_on: 2,
                received_count: 2,
            }
        );
        // Verify state of idty 1
        assert_eq!(
            DefaultCertification::idty_cert_meta(1),
            crate::IdtyCertMeta {
                issued_count: 2,
                next_issuable_on: 0,
                received_count: 2,
            }
        );
        // Verify state of idty 2
        assert_eq!(
            DefaultCertification::idty_cert_meta(2),
            crate::IdtyCertMeta {
                issued_count: 2,
                next_issuable_on: 1,
                received_count: 2,
            }
        );
        // Cert 2->1 must be removable at block #3
        assert_eq!(
            DefaultCertification::certs_removable_on(3),
            Some(vec![(2, 1)]),
        );

        run_to_block(3);
        // Cert 2->1 must have expired
        assert_eq!(
            System::events()[0].event,
            RuntimeEvent::DefaultCertification(Event::RemovedCert {
                issuer: 2,
                issuer_issued_count: 1,
                receiver: 1,
                receiver_received_count: 1,
                expiration: true,
            },)
        );
    });
}

#[test]
fn test_cert_period() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => 10,
                2 => 10,
            ],
            1 => btreemap![
                0 => 10,
                2 => 10,
            ],
            2 => btreemap![
                0 => 10,
                1 => 10,
            ],
        ],
    })
    .execute_with(|| {
        assert_eq!(
            DefaultCertification::idty_cert_meta(0),
            crate::IdtyCertMeta {
                issued_count: 2,
                next_issuable_on: 2,
                received_count: 2,
            }
        );
        assert_eq!(
            DefaultCertification::add_cert(Origin::signed(0), 0, 3),
            Err(Error::<Test, _>::NotRespectCertPeriod.into())
        );
        run_to_block(CertPeriod::get());
        assert_ok!(DefaultCertification::add_cert(Origin::signed(0), 0, 3));
        run_to_block(CertPeriod::get() + 1);
        assert_eq!(
            DefaultCertification::add_cert(Origin::signed(0), 0, 4),
            Err(Error::<Test, _>::NotRespectCertPeriod.into())
        );
        run_to_block((2 * CertPeriod::get()) + 1);
        assert_ok!(DefaultCertification::add_cert(Origin::signed(0), 0, 4));
    });
}
