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

    fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event.into());
    }

    #[benchmark]
    fn go_offline() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));

        assert_has_event::<T>(Event::<T>::MemberGoOffline { member: id }.into());
    }

    #[benchmark]
    fn go_online() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        let caller: T::AccountId = Members::<T>::get(id).unwrap().owner_key;
        OnlineAuthorities::<T>::mutate(|ids| {
            ids.retain(|&x| x != id);
        });
        OutgoingAuthorities::<T>::mutate(|ids| {
            ids.retain(|&x| x != id);
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

        #[extrinsic_call]
        _(RawOrigin::Root, id);

        assert_has_event::<T>(Event::<T>::MemberRemoved { member: id }.into());
    }

    #[benchmark]
    fn remove_member_from_blacklist() {
        let id: T::MemberId = OnlineAuthorities::<T>::get()[0];
        Blacklist::<T>::mutate(|blacklist| {
            blacklist.push(id);
        });

        #[extrinsic_call]
        _(RawOrigin::Root, id);

        assert_has_event::<T>(Event::<T>::MemberRemovedFromBlacklist { member: id }.into());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(2), crate::mock::Test);
}
