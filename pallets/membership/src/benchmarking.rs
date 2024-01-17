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

use frame_benchmarking::benchmarks;
use frame_system::pallet_prelude::BlockNumberFor;

#[cfg(test)]
use maplit::btreemap;

use crate::Pallet;

// fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
//     frame_system::Pallet::<T>::assert_has_event(generic_event.into());
// }

benchmarks! {
    where_clause {
        where
            T::IdtyId: From<u32>,
            <T as frame_system::Config>::BlockNumber: From<u32>,
    }

    // TODO membership add and renewal should be included to distance on_new_session as worst case scenario

    // Base weight of an empty initialize
    on_initialize {
    }: {Pallet::<T>::on_initialize(BlockNumberFor::<T>::zero());}

    expire_memberships {
        let i in 0..3; // Limited by the number of validators
        // Arbitrarily high, to be in the worst case of wot instance,
        // this will overcount the weight in hooks see https://git.duniter.org/nodes/rust/duniter-v2s/-/issues/167
        let block_number: BlockNumberFor::<T> = 10_000_000.into();
        frame_system::pallet::Pallet::<T>::set_block_number(block_number);
        let mut idties: Vec<T::IdtyId> = Vec::new();
        for j in 1..i+1 {
            let j: T::IdtyId = j.into();
            Membership::<T>::insert(j, MembershipData::<T::BlockNumber>::default());
            idties.push(j);
        }
        MembershipsExpireOn::<T>::insert(block_number, idties);
        assert_eq!(MembershipsExpireOn::<T>::get(block_number).len(), i as usize);
    }: {Pallet::<T>::expire_memberships(block_number);}
    verify {
        assert_eq!(MembershipsExpireOn::<T>::get(block_number).len(), 0_usize);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::MembershipConfig {
        memberships: btreemap![
            3 => MembershipData {
                expire_on: 3,
            },
        ],}),
        crate::mock::Test
    );
}
