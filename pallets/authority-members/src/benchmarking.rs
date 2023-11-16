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

use crate::Pallet;

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T::MemberId: From<u32>,
    }
    go_offline {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin)
    verify {
        assert_has_event::<T>(Event::<T>::MemberGoOffline(id).into());
    }
     go_online {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        OnlineAuthorities::<T>::mutate(|ids| {
            ids.retain(|&x| x != id);
        });
        OutgoingAuthorities::<T>::mutate(|ids| {
            ids.retain(|&x| x != id);
        });
    }: _<T::RuntimeOrigin>(caller_origin)
    verify {
        assert_has_event::<T>(Event::<T>::MemberGoOnline(id).into());
    }
     set_session_keys {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
            let validator_id = T::ValidatorIdOf::convert(caller.clone()).unwrap();
            let session_keys: T::Keys = pallet_session::NextKeys::<T>::get(validator_id).unwrap().into();
        }: _<T::RuntimeOrigin>(caller_origin, session_keys)
     remove_member {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller_origin = RawOrigin::Root.into();
        }: _<T::RuntimeOrigin>(caller_origin, id.clone())
    verify {
        assert_has_event::<T>(Event::<T>::MemberRemoved(id).into());
    }
     remove_member_from_blacklist {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        BlackList::<T>::mutate(|blacklist| {
            blacklist.push(id);
        });
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), id)
    verify {
        assert_has_event::<T>(Event::<T>::MemberRemovedFromBlackList(id).into());
    }

     impl_benchmark_test_suite!(
            Pallet,
            crate::mock::new_test_ext(2),
            crate::mock::Test
        );
}
