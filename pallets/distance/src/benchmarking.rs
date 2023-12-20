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

use codec::Encode;
use frame_benchmarking::{benchmarks, vec};
use frame_support::traits::{Currency, OnFinalize};
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;
use sp_runtime::traits::{Bounded, One};
use sp_runtime::Perbill;

use crate::Pallet;

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn populate_pool<T: Config>(i: u32) -> Result<(), &'static str> {
    EvaluationPool0::<T>::mutate(|current_pool| -> Result<(), &'static str> {
        for j in 0..i {
            current_pool
                .evaluations
                .try_push((j, median::MedianAcc::new()))
                .map_err(|_| Error::<T>::QueueFull)?;
        }
        Ok(())
    })
}

benchmarks! {
    where_clause {
        where
        T: pallet_balances::Config, T::Balance: From<u64>,
        T::BlockNumber: From<u32>,
    }
    request_distance_evaluation {
            let idty = T::IdtyIndex::one();
            let caller: T::AccountId  = pallet_identity::Identities::<T>::get(idty).unwrap().owner_key;
            let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
            let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());
    }: _<T::RuntimeOrigin>(caller_origin.clone())
    verify {
        assert!(IdentityDistanceStatus::<T>::get(idty) == Some((caller.clone(), DistanceStatus::Pending)), "Request not added");
        assert_has_event::<T>(Event::<T>::EvaluationRequested { idty_index: idty, who: caller }.into());
    }
    update_evaluation {
        let digest_data = sp_consensus_babe::digests::PreDigest::SecondaryPlain(
        sp_consensus_babe::digests::SecondaryPlainPreDigest { authority_index: 0u32, slot: Default::default() });
        // A BABE digest item is needed to check authorship
        let digest = sp_runtime::DigestItem::PreRuntime(*b"BABE", digest_data.encode());
        let _ = <frame_system::Pallet<T>>::deposit_log(digest);
        let idty = T::IdtyIndex::one();
        let caller: T::AccountId  = pallet_identity::Identities::<T>::get(idty).unwrap().owner_key;
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let i in 1 .. MAX_EVALUATIONS_PER_SESSION => populate_pool::<T>(i)?;
    }: _<T::RuntimeOrigin>(RawOrigin::None.into(), ComputationResult{distances: vec![Perbill::one(); i as usize]})
    verify {
        assert_has_event::<T>(Event::<T>::EvaluationUpdated { evaluator: caller }.into());
    }
    force_update_evaluation {
            let idty = T::IdtyIndex::one();
            let caller: T::AccountId  = pallet_identity::Identities::<T>::get(idty).unwrap().owner_key;
            let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
            let i in 1 .. MAX_EVALUATIONS_PER_SESSION => populate_pool::<T>(i)?;
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), caller.clone(), ComputationResult{distances: vec![Perbill::one(); i as usize]})
    verify {
        assert_has_event::<T>(Event::<T>::EvaluationUpdated { evaluator: caller }.into());
    }
    force_set_distance_status {
            let idty = T::IdtyIndex::one();
            let caller: T::AccountId  = pallet_identity::Identities::<T>::get(idty).unwrap().owner_key;
            let status = Some((caller.clone(), DistanceStatus::Valid));
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), idty, status.clone())
    verify {
        assert!(IdentityDistanceStatus::<T>::get(idty) == Some((caller, DistanceStatus::Valid)), "Status not set");
        assert_has_event::<T>(Event::<T>::EvaluationStatusForced { idty_index: idty, status }.into());
    }
    on_finalize {
        DidUpdate::<T>::set(true);
    }: { Pallet::<T>::on_finalize(Default::default()); }
    verify {
        assert!(!DidUpdate::<T>::get());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
