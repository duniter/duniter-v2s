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

use crate::mock::*;
use crate::Error;
use frame_support::assert_ok;
//use frame_system::{EventRecord, Phase};
use maplit::{btreemap, btreeset};
use sp_std::collections::btree_map::BTreeMap;

#[test]
fn test_must_receive_cert_before_can_issue() {
    new_test_ext(DefaultCertificationConfig {
        certs_by_issuer: BTreeMap::new(),
        phantom: core::marker::PhantomData,
    })
    .execute_with(|| {
        assert_eq!(
            DefaultCertification::add_cert(Origin::root(), 0, 1),
            Err(Error::<Test, _>::IdtyMustReceiveCertsBeforeCanIssue.into())
        );
    });
}

#[test]
fn test_cert_period() {
    new_test_ext(DefaultCertificationConfig {
        certs_by_issuer: btreemap![0 => btreeset![1]],
        phantom: core::marker::PhantomData,
    })
    .execute_with(|| {
        assert_eq!(
            DefaultCertification::add_cert(Origin::root(), 0, 2),
            Err(Error::<Test, _>::NotRespectCertPeriod.into())
        );
        run_to_block(CertPeriod::get());
        assert_ok!(DefaultCertification::add_cert(Origin::root(), 0, 2));
        run_to_block(CertPeriod::get() + 1);
        assert_eq!(
            DefaultCertification::add_cert(Origin::root(), 0, 3),
            Err(Error::<Test, _>::NotRespectCertPeriod.into())
        );
        run_to_block((2 * CertPeriod::get()) + 1);
        assert_ok!(DefaultCertification::add_cert(Origin::root(), 0, 3));
    });
}

#[test]
fn test_renewable_period() {
    new_test_ext(DefaultCertificationConfig {
        certs_by_issuer: btreemap![0 => btreeset![1]],
        phantom: core::marker::PhantomData,
    })
    .execute_with(|| {
        run_to_block(CertPeriod::get());
        assert_eq!(
            DefaultCertification::add_cert(Origin::root(), 0, 1),
            Err(Error::<Test, _>::NotRespectRenewablePeriod.into())
        );
        run_to_block(RenewablePeriod::get());
        assert_ok!(DefaultCertification::add_cert(Origin::root(), 0, 1));
        run_to_block(RenewablePeriod::get() + CertPeriod::get());
        assert_eq!(
            DefaultCertification::add_cert(Origin::root(), 0, 1),
            Err(Error::<Test, _>::NotRespectRenewablePeriod.into())
        );
        run_to_block((2 * RenewablePeriod::get()) + 1);
        assert_ok!(DefaultCertification::add_cert(Origin::root(), 0, 1));
    });
}
