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

use frame_benchmarking::benchmarks_instance_pallet;
use frame_benchmarking::Zero;
use frame_system::RawOrigin;
use sp_runtime::traits::Convert;

#[cfg(test)]
use maplit::btreemap;

use crate::Pallet;

fn assert_has_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn add_certs<T: Config<I>, I: 'static>(i: u32, receiver: T::IdtyIndex) -> Result<(), &'static str> {
    Pallet::<T, I>::remove_all_certs_received_by(RawOrigin::Root.into(), receiver)?;
    for j in 1..i {
        Pallet::<T, I>::do_add_cert_checked(j.into(), receiver, false)?;
    }
    assert!(
        CertsByReceiver::<T, I>::get(receiver).len() as u32 == i - 1,
        "Certs not added",
    );
    Ok(())
}

benchmarks_instance_pallet! {
    where_clause {
        where
            T::IdtyIndex: From<u32>,
    }
    add_cert {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId  = T::OwnerKeyOf::convert(issuer).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let receiver: T::IdtyIndex = 2.into();
        Pallet::<T, I>::del_cert(RawOrigin::Root.into(), issuer, receiver)?;
        let issuer_cert: u32 = StorageIdtyCertMeta::<T, I>::get(issuer).issued_count;
        let receiver_cert: u32 = StorageIdtyCertMeta::<T, I>::get(receiver).received_count;
        frame_system::pallet::Pallet::<T>::set_block_number(T::CertPeriod::get());
    }: _<T::RuntimeOrigin>(caller_origin, issuer, receiver)
    verify {
        assert_has_event::<T, I>(Event::<T, I>::CertAdded{ issuer, receiver }.into());
    }
    del_cert {
        let issuer: T::IdtyIndex = 1.into();
        let receiver: T::IdtyIndex = 0.into();
        Pallet::<T, I>::do_add_cert_checked(issuer, receiver, false)?;
        let receiver_cert: u32 = StorageIdtyCertMeta::<T, I>::get(receiver).received_count;
        let issuer_cert: u32 = StorageIdtyCertMeta::<T, I>::get(issuer).issued_count;
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), issuer, receiver)
    verify {
        assert_has_event::<T, I>(Event::<T, I>::CertRemoved{ issuer,  receiver, expiration: false }.into());
    }
    remove_all_certs_received_by {
        let receiver: T::IdtyIndex = 0.into();
        let i in 2..1000 => add_certs::<T, I>(i, receiver)?;
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(),  receiver)
    verify {
        assert!(CertsByReceiver::<T, I>::get(receiver).is_empty() );
    }
    on_initialize {
        assert!(StorageCertsRemovableOn::<T, I>::try_get(T::BlockNumber::zero()).is_err());
    }: {Pallet::<T, I>::on_initialize(T::BlockNumber::zero());}
    do_remove_cert_noop {
    }: {Pallet::<T, I>::do_remove_cert(100.into(), 101.into(), Some(T::BlockNumber::zero()));}
    do_remove_cert {
        let issuer: T::IdtyIndex = 1.into();
        let receiver: T::IdtyIndex = 0.into();
        Pallet::<T, I>::do_add_cert_checked(issuer, receiver, false)?;
        let issuer_cert: u32 = StorageIdtyCertMeta::<T, I>::get(issuer).issued_count;
        let receiver_cert: u32 = StorageIdtyCertMeta::<T, I>::get(receiver).received_count;
        let block_number = T::ValidityPeriod::get();
        frame_system::pallet::Pallet::<T>::set_block_number(block_number);
    }: {Pallet::<T, I>::do_remove_cert(issuer, receiver, Some(block_number));}
    verify {
        assert_has_event::<T, I>(Event::<T, I>::CertRemoved{ issuer,  receiver, expiration: true }.into());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::DefaultCertificationConfig {
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
            ] ,
        }),
        crate::mock::Test
    );
}
