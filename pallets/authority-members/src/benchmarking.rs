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
use crate::Pallet;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks(
        where
            <T as Config>::MemberId: From<u32>,
)]
mod benchmarks {
    use super::*;
    use scale_info::prelude::vec::Vec;

    fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event);
    }

    fn dense_member_ids_excluding<T: Config>(
        start: u32,
        len: u32,
        excluded: &T::MemberId,
    ) -> Vec<T::MemberId>
    where
        <T as Config>::MemberId: From<u32> + Ord,
    {
        let target = len as usize;
        let mut ids: Vec<T::MemberId> = (start..start.saturating_add(len.saturating_add(1)))
            .map(Into::into)
            .filter(|id| id != excluded)
            .take(target)
            .collect();
        ids.sort();
        ids
    }

    #[benchmark]
    fn go_offline() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        let max = T::MaxAuthorities::get().max(2);
        // Stress insertion into OutgoingAuthorities while keeping the not-incoming branch.
        IncomingAuthorities::<T>::kill();
        OutgoingAuthorities::<T>::put(dense_member_ids_excluding::<T>(10_000, max - 1, &id));

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));

        assert_has_event::<T>(Event::<T>::MemberGoOffline { member: id }.into());
    }

    #[benchmark]
    fn go_online() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        let max = T::MaxAuthorities::get().max(2);
        // Keep dense sorted sets with explicit inclusion/exclusion of `id`.
        OnlineAuthorities::<T>::mutate(|ids| {
            ids.clear();
            ids.extend(dense_member_ids_excluding::<T>(30_000, max - 1, &id));
        });
        OutgoingAuthorities::<T>::mutate(|ids| {
            ids.clear();
            ids.extend(dense_member_ids_excluding::<T>(10_000, max - 1, &id));
        });
        IncomingAuthorities::<T>::mutate(|ids| {
            ids.clear();
            ids.extend(dense_member_ids_excluding::<T>(20_000, max - 1, &id));
        });

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));

        assert_has_event::<T>(Event::<T>::MemberGoOnline { member: id }.into());
    }

    #[benchmark]
    fn set_session_keys() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        let validator_id = T::ValidatorIdOf::convert(caller.clone()).unwrap();
        let session_keys: T::Keys = pallet_session::NextKeys::<T>::get(validator_id).unwrap();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), session_keys);
    }

    #[benchmark]
    fn remove_member() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let max = T::MaxAuthorities::get().max(2);
        // Stress all removal helpers with dense vectors.
        OnlineAuthorities::<T>::mutate(|ids| {
            ids.clear();
            ids.extend(dense_member_ids_excluding::<T>(10_000, max - 1, &id));
            ids.push(id);
            ids.sort();
        });
        IncomingAuthorities::<T>::mutate(|ids| {
            ids.clear();
            ids.extend(dense_member_ids_excluding::<T>(20_000, max - 1, &id));
            ids.push(id);
            ids.sort();
        });
        OutgoingAuthorities::<T>::put(dense_member_ids_excluding::<T>(30_000, max - 1, &id));

        #[extrinsic_call]
        _(RawOrigin::Root, id);

        assert_has_event::<T>(Event::<T>::MemberRemoved { member: id }.into());
    }

    #[benchmark]
    fn remove_member_from_blacklist() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let max = T::MaxAuthorities::get().max(2);
        // Keep blacklist dense and sorted so binary_search/remove always exercise the intended path.
        Blacklist::<T>::mutate(|blacklist| {
            blacklist.clear();
            blacklist.extend(dense_member_ids_excluding::<T>(10_000, max - 1, &id));
            blacklist.push(id);
            blacklist.sort();
        });

        #[extrinsic_call]
        _(RawOrigin::Root, id);

        assert_has_event::<T>(Event::<T>::MemberRemovedFromBlacklist { member: id }.into());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(4), crate::mock::Test);
}
