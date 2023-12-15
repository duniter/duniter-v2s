// Copyright 2021-2022 Axiom-Team
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
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::RawOrigin;
use sp_runtime::traits::{Convert, One};

#[cfg(test)]
use maplit::btreemap;

use crate::Pallet;

fn assert_has_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks_instance_pallet! {
    where_clause {
        where
            T::IdtyId: From<u32>,
    }

    // claim membership
    claim_membership {
        let idty: T::IdtyId = 3.into();
        Membership::<T, I>::take(idty);
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        T::BenchmarkSetupHandler::force_status_ok(&idty, &caller);
    }: _<T::RuntimeOrigin>(caller_origin)
    verify {
        assert_has_event::<T, I>(Event::<T, I>::MembershipAdded{member: idty, expire_on: BlockNumberFor::<T>::one() + T::MembershipPeriod::get()}.into());
    }

    // renew membership
    renew_membership {
        let idty: T::IdtyId = 3.into();
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        T::BenchmarkSetupHandler::force_status_ok(&idty, &caller);
    }: _<T::RuntimeOrigin>(caller_origin)
    verify {
        assert_has_event::<T, I>(Event::<T, I>::MembershipAdded{member: idty, expire_on: BlockNumberFor::<T>::one() + T::MembershipPeriod::get()}.into());
    }

    // revoke membership
    revoke_membership {
        let idty: T::IdtyId = 3.into();
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin)
    verify {
        assert_has_event::<T, I>(Event::<T, I>::MembershipRemoved{member: idty, reason: MembershipRemovalReason::Revoked}.into());
    }

    // Base weight of an empty initialize
    on_initialize {
    }: {Pallet::<T, I>::on_initialize(BlockNumberFor::<T>::zero());}

    expire_memberships {
        let i in 0..1024;
        let mut idties: Vec<T::IdtyId> = Vec::new();
        for j in 0..i {
            let j: T::IdtyId = j.into();
            Membership::<T, I>::insert(j, MembershipData::<T::BlockNumber>::default());
            idties.push(j);
        }
        MembershipsExpireOn::<T, I>::insert(BlockNumberFor::<T>::zero(), idties);
        assert_eq!(MembershipsExpireOn::<T, I>::get(BlockNumberFor::<T>::zero()).len(), i as usize);
    }: {Pallet::<T, I>::expire_memberships(BlockNumberFor::<T>::zero());}
    verify {
        assert_eq!(MembershipsExpireOn::<T, I>::get(BlockNumberFor::<T>::zero()).len(), 0 as usize);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::DefaultMembershipConfig {
        memberships: btreemap![
            3 => MembershipData {
                expire_on: 3,
            },
        ],}),
        crate::mock::Test
    );
}
