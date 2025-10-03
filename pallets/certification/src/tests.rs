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

use crate::{Error, Event, mock::*};
use frame_support::{assert_noop, assert_ok};
use maplit::btreemap;
use scale_info::prelude::{collections::BTreeMap, vec};

#[test]
fn test_must_receive_cert_before_can_issue() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: BTreeMap::new(),
    })
    .execute_with(|| {
        assert_eq!(
            DefaultCertification::add_cert(RuntimeOrigin::signed(0), 1),
            Err(Error::<Test>::NotEnoughCertReceived.into())
        );
    });
}

#[test]
fn test_cannot_certify_self() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => Some(5),
                2 => Some(5),
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(2);

        assert_eq!(
            DefaultCertification::add_cert(RuntimeOrigin::signed(0), 0),
            Err(Error::<Test>::CannotCertifySelf.into())
        );
    });
}

#[test]
fn test_genesis_build() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => Some(7),
                2 => Some(9),
            ],
            1 => btreemap![
                0 => Some(10),
                2 => Some(3),
            ],
            2 => btreemap![
                0 => Some(5),
                1 => Some(4),
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
            RuntimeEvent::DefaultCertification(Event::CertRemoved {
                issuer: 2,
                receiver: 1,
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
                1 => Some(10),
                2 => Some(10),
            ],
            1 => btreemap![
                0 => Some(10),
                2 => Some(10),
            ],
            2 => btreemap![
                0 => Some(10),
                1 => Some(10),
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
            DefaultCertification::add_cert(RuntimeOrigin::signed(0), 3),
            Err(Error::<Test>::NotRespectCertPeriod.into())
        );
        run_to_block(CertPeriod::get());
        assert_ok!(DefaultCertification::add_cert(RuntimeOrigin::signed(0), 3));
        run_to_block(CertPeriod::get() + 1);
        assert_eq!(
            DefaultCertification::add_cert(RuntimeOrigin::signed(0), 4),
            Err(Error::<Test>::NotRespectCertPeriod.into())
        );
        run_to_block((2 * CertPeriod::get()) + 1);
        assert_ok!(DefaultCertification::add_cert(RuntimeOrigin::signed(0), 4));
    });
}

// after given validity period, a certification should expire
#[test]
fn test_cert_expiry() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: true,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => Some(5),
                2 => Some(5),
            ],
            1 => btreemap![
                0 => Some(6),
                2 => Some(6),
            ],
            2 => btreemap![
                0 => Some(7),
                1 => Some(7),
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(5);
        // Expiry of cert by issuer 1
        System::assert_has_event(RuntimeEvent::DefaultCertification(Event::CertRemoved {
            issuer: 1,
            receiver: 0,
            expiration: true,
        }));
        // Expiry of cert by issuer 2
        System::assert_has_event(RuntimeEvent::DefaultCertification(Event::CertRemoved {
            receiver: 0,
            issuer: 2,
            expiration: true,
        }));
    });
}

// when renewing a certification, it should not expire now, but later
#[test]
fn test_cert_renewal() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: false,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => Some(5),
                2 => Some(20),
            ],
            1 => btreemap![
                0 => Some(20),
                2 => Some(20),
            ],
            2 => btreemap![
                0 => Some(20),
                1 => Some(20),
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(2);
        // renew certification from bob to alice
        // this certification should expire 10 blocks later (at block 12)
        assert_eq!(
            DefaultCertification::renew_cert(RuntimeOrigin::signed(1), 0),
            Ok(())
        );
        System::assert_last_event(RuntimeEvent::DefaultCertification(Event::CertRenewed {
            issuer: 1,
            receiver: 0,
        }));

        run_to_block(12);
        // expiry of previously renewed cert
        System::assert_last_event(RuntimeEvent::DefaultCertification(Event::CertRemoved {
            issuer: 1,
            receiver: 0,
            expiration: true,
        }));
    });
}

// when renewing a certification, issuer should not be able to emit a new cert before certification delay
#[test]
fn test_cert_renewal_cert_delay() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: false,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => Some(5),
                2 => Some(20),
            ],
            1 => btreemap![
                0 => Some(20),
                2 => Some(20),
            ],
            2 => btreemap![
                0 => Some(20),
                1 => Some(20),
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(2);
        // renew certification from bob to alice
        assert_eq!(
            DefaultCertification::renew_cert(RuntimeOrigin::signed(1), 0),
            Ok(())
        );
        System::assert_last_event(RuntimeEvent::DefaultCertification(Event::CertRenewed {
            issuer: 1,
            receiver: 0,
        }));

        run_to_block(3);
        // try to renew again
        assert_noop!(
            DefaultCertification::add_cert(RuntimeOrigin::signed(1), 0),
            Error::<Test>::NotRespectCertPeriod,
        );
        // no renewal event should be emitted
        assert_eq!(System::events().last(), None);
    });
}

// when renewing a certification, the certification should not expire before new expiration
#[test]
fn test_cert_renewal_expiration() {
    new_test_ext(DefaultCertificationConfig {
        apply_cert_period_at_genesis: false,
        certs_by_receiver: btreemap![
            0 => btreemap![
                1 => Some(5),
                2 => Some(20),
            ],
            1 => btreemap![
                0 => Some(20),
                2 => Some(20),
            ],
            2 => btreemap![
                0 => Some(20),
                1 => Some(20),
            ],
        ],
    })
    .execute_with(|| {
        run_to_block(2);
        // renew certification from bob to alice
        // this certification should expire 10 blocks later (at block 12)
        assert_eq!(
            DefaultCertification::renew_cert(RuntimeOrigin::signed(1), 0),
            Ok(())
        );
        System::assert_last_event(RuntimeEvent::DefaultCertification(Event::CertRenewed {
            issuer: 1,
            receiver: 0,
        }));

        run_to_block(4);
        // renew certification from bob to alice again
        // this certification should expire 10 blocks later (at block 14)
        assert_eq!(
            DefaultCertification::renew_cert(RuntimeOrigin::signed(1), 0),
            Ok(())
        );
        System::assert_last_event(RuntimeEvent::DefaultCertification(Event::CertRenewed {
            issuer: 1,
            receiver: 0,
        }));

        // no certification should expire at these blocks
        // hint : prune_certifications checks that the certification has not been renewed
        run_to_block(12);
        assert_eq!(System::events().last(), None);

        run_to_block(14);
        // expiry of previously renewed cert
        System::assert_last_event(RuntimeEvent::DefaultCertification(Event::CertRemoved {
            issuer: 1,
            receiver: 0,
            expiration: true,
        }));
    });
}
