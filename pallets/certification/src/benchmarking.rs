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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;

use crate::Pallet;

#[benchmarks(
        where
            T::IdtyIndex: From<u32>,
)]
mod benchmarks {
    use super::*;

    fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event);
    }

    fn add_certs<T: Config>(i: u32, receiver: T::IdtyIndex) -> Result<(), &'static str> {
        Pallet::<T>::remove_all_certs_received_by(RawOrigin::Root.into(), receiver)?;
        for j in 1..i + 1 {
            Pallet::<T>::do_add_cert_checked(j.into(), receiver, false)?;
        }
        assert!(
            CertsByReceiver::<T>::get(receiver).len() as u32 == i,
            "Certs not added",
        );
        Ok(())
    }

    #[benchmark]
    fn add_cert() -> Result<(), BenchmarkError> {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        let receiver: T::IdtyIndex = 2.into();
        Pallet::<T>::del_cert(RawOrigin::Root.into(), issuer, receiver)?;
        frame_system::pallet::Pallet::<T>::set_block_number(T::CertPeriod::get());

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), receiver);

        assert_has_event::<T>(Event::<T>::CertAdded { issuer, receiver }.into());
        Ok(())
    }

    #[benchmark]
    fn renew_cert() -> Result<(), BenchmarkError> {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        let receiver: T::IdtyIndex = 2.into();
        Pallet::<T>::del_cert(RawOrigin::Root.into(), issuer, receiver)?;
        frame_system::pallet::Pallet::<T>::set_block_number(T::CertPeriod::get());
        Pallet::<T>::add_cert(RawOrigin::Signed(caller.clone()).into(), receiver)?;
        frame_system::pallet::Pallet::<T>::set_block_number(
            T::CertPeriod::get() + T::CertPeriod::get(),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), receiver);

        assert_has_event::<T>(Event::<T>::CertAdded { issuer, receiver }.into());
        Ok(())
    }

    #[benchmark]
    fn del_cert() {
        let issuer: T::IdtyIndex = 1.into();
        let receiver: T::IdtyIndex = 2.into();
        // try to add cert if missing, else ignore
        // this depends on initial data
        let _ = Pallet::<T>::do_add_cert_checked(issuer, receiver, false);

        #[extrinsic_call]
        _(RawOrigin::Root, issuer, receiver);

        assert_has_event::<T>(
            Event::<T>::CertRemoved {
                issuer,
                receiver,
                expiration: false,
            }
            .into(),
        );
    }

    #[benchmark]
    fn remove_all_certs_received_by(i: Linear<2, 1_000>) -> Result<(), BenchmarkError> {
        let receiver: T::IdtyIndex = 0.into();
        add_certs::<T>(i, receiver)?;

        #[extrinsic_call]
        _(RawOrigin::Root, receiver);

        assert!(CertsByReceiver::<T>::get(receiver).is_empty());
        Ok(())
    }

    #[benchmark]
    fn on_initialize() {
        assert!(CertsRemovableOn::<T>::try_get(BlockNumberFor::<T>::zero()).is_err());

        #[block]
        {
            Pallet::<T>::on_initialize(BlockNumberFor::<T>::zero());
        }
    }

    #[benchmark]
    fn do_remove_cert_noop() {
        #[block]
        {
            Pallet::<T>::do_remove_cert(100.into(), 101.into(), Some(BlockNumberFor::<T>::zero()));
        }
    }

    #[benchmark]
    fn do_remove_cert() -> Result<(), BenchmarkError> {
        let issuer: T::IdtyIndex = 1.into();
        let receiver: T::IdtyIndex = 0.into();
        Pallet::<T>::do_remove_cert(issuer, receiver, None);
        Pallet::<T>::do_add_cert_checked(issuer, receiver, false)?;
        let block_number = T::ValidityPeriod::get();
        frame_system::pallet::Pallet::<T>::set_block_number(block_number);

        #[block]
        {
            Pallet::<T>::do_remove_cert(issuer, receiver, Some(block_number));
        }

        assert_has_event::<T>(
            Event::<T>::CertRemoved {
                issuer,
                receiver,
                expiration: true,
            }
            .into(),
        );
        Ok(())
    }

    #[benchmark]
    fn do_remove_all_certs_received_by(i: Linear<2, 100>) -> Result<(), BenchmarkError> {
        let receiver: T::IdtyIndex = 0.into();
        add_certs::<T>(i, receiver)?;

        #[block]
        {
            Pallet::<T>::do_remove_all_certs_received_by(receiver);
        }

        for issuer in 1..i + 1 {
            assert_has_event::<T>(
                Event::<T>::CertRemoved {
                    issuer: issuer.into(),
                    receiver,
                    expiration: false,
                }
                .into(),
            );
        }
        Ok(())
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::DefaultCertificationConfig {
            apply_cert_period_at_genesis: true,
            certs_by_receiver: maplit::btreemap![
                0 => maplit::btreemap![
                    1 => Some(7),
                    2 => Some(9),
                ],
                1 => maplit::btreemap![
                    0 => Some(10),
                    2 => Some(3),
                ],
            ],
        }),
        crate::mock::Test
    );
}
