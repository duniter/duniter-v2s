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

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_runtime::traits::Convert;

#[cfg(test)]
use maplit::btreemap;

use crate::Pallet;

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T::IdtyIndex: From<u32>
    }
    invite_smith {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::OwnerKeyOf::convert(issuer).unwrap();
        Pallet::<T>::on_smith_goes_online(1.into());
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let receiver: T::IdtyIndex = 4.into();
    }: _<T::RuntimeOrigin>(caller_origin, receiver)
    verify {
        assert_has_event::<T>(Event::<T>::InvitationSent{
            idty_index: receiver,
            invited_by: issuer,
        }.into());
    }
    accept_invitation {
        // Worst case preparation
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::OwnerKeyOf::convert(issuer).unwrap();
        Pallet::<T>::on_smith_goes_online(1.into());
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let receiver: T::IdtyIndex = 4.into();
        Pallet::<T>::invite_smith(caller_origin, receiver)?;
        // test
        let issuer: T::IdtyIndex = 4.into();
        let caller: T::AccountId = T::OwnerKeyOf::convert(issuer).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin)
    verify {
        assert_has_event::<T>(Event::<T>::InvitationAccepted{
            idty_index: receiver,
        }.into());
    }
    certify_smith {
        // Worst case preparation
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::OwnerKeyOf::convert(issuer).unwrap();
        Pallet::<T>::on_smith_goes_online(1.into());
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let receiver: T::IdtyIndex = 4.into();
        Pallet::<T>::invite_smith(caller_origin, receiver)?;
        let issuer: T::IdtyIndex = receiver;
        let caller: T::AccountId = T::OwnerKeyOf::convert(issuer).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        Pallet::<T>::accept_invitation(caller_origin)?;
        // test
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::OwnerKeyOf::convert(issuer).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin, receiver)
    verify {
        assert_has_event::<T>(Event::<T>::CertificationReceived{
            idty_index: receiver,
            issued_by: issuer,
        }.into());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(GenesisConfig {
            initial_smiths: btreemap![
                1 => (false, vec![2, 3]),
                2 => (false, vec![1, 3]),
                3 => (false, vec![1, 2]),
            ],
        }),
        crate::mock::Runtime
    );
}
