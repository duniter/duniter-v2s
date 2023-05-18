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
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_runtime::traits::Convert;

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
    request_membership {
        // Dave identity (4)
        // for main wot, no constraints
        // for smith subwot, his pubkey is hardcoded in default metadata
        let idty: T::IdtyId = 4.into();
        Membership::<T, I>::take(idty);
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        // Lazily prepare call as this extrinsic will always return an errror when in subwot
        let call = Call::<T, I>::request_membership { metadata: T::MetaData ::default()};
    }: {
        call.dispatch_bypass_filter(caller_origin).ok();
    }
    verify {
        if T::CheckMembershipCallAllowed::check_idty_allowed_to_request_membership(&idty).is_ok() {
            assert_has_event::<T, I>(Event::<T, I>::MembershipRequested(idty).into());
        }
    }
    claim_membership {
        let idty: T::IdtyId = 3.into();
        Membership::<T, I>::take(idty);
        PendingMembership::<T, I>::insert(idty.clone(), T::MetaData::default());
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin, Some(idty))
    verify {
        assert_has_event::<T, I>(Event::<T, I>::MembershipAcquired(idty).into());
    }
    renew_membership {
        let idty: T::IdtyId = 3.into();
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin, Some(idty))
    verify {
        assert_has_event::<T, I>(Event::<T, I>::MembershipRenewed(idty).into());
    }
    revoke_membership {
        let idty: T::IdtyId = 3.into();
        let caller: T::AccountId = T::AccountIdOf::convert(idty.clone()).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
    }: _<T::RuntimeOrigin>(caller_origin, Some(idty))
    verify {
        assert_has_event::<T, I>(Event::<T, I>::MembershipRevoked(idty).into());
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
