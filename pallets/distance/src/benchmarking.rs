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
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, OnFinalize};
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;
use sp_runtime::traits::{Bounded, One};
use sp_runtime::Perbill;

use crate::Pallet;

#[benchmarks(
        where
        T: pallet_balances::Config,
		T::Balance: From<u64>,
        BlockNumberFor<T>: From<u32>,
)]
mod benchmarks {
    use super::*;

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

    #[benchmark]
    fn request_distance_evaluation() {
        let idty = T::IdtyIndex::one();
        let caller: T::AccountId = pallet_identity::Identities::<T>::get(idty)
            .unwrap()
            .owner_key;
        let _ =
            <Balances<T> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()));

        assert!(
            PendingEvaluationRequest::<T>::get(idty) == Some(caller.clone()),
            "Request not added"
        );
        assert_has_event::<T>(
            Event::<T>::EvaluationRequested {
                idty_index: idty,
                who: caller,
            }
            .into(),
        );
    }

    #[benchmark]
    fn request_distance_evaluation_for() {
        let idty = T::IdtyIndex::one();
        let caller: T::AccountId = pallet_identity::Identities::<T>::get(idty)
            .unwrap()
            .owner_key;
        <Balances<T> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());
        let target: T::IdtyIndex = 2u32;
        // set target status since targeted distance evaluation only allowed for unvalidated
        pallet_identity::Identities::<T>::mutate(target, |idty_val| {
            idty_val.as_mut().unwrap().status = pallet_identity::IdtyStatus::Unvalidated
        });

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), target);

        assert!(
            PendingEvaluationRequest::<T>::get(target) == Some(caller.clone()),
            "Request not added"
        );
        assert_has_event::<T>(
            Event::<T>::EvaluationRequested {
                idty_index: target,
                who: caller,
            }
            .into(),
        );
    }

    #[benchmark]
    fn update_evaluation(i: Linear<1, MAX_EVALUATIONS_PER_SESSION>) -> Result<(), BenchmarkError> {
        let digest_data = sp_consensus_babe::digests::PreDigest::SecondaryPlain(
            sp_consensus_babe::digests::SecondaryPlainPreDigest {
                authority_index: 0u32,
                slot: Default::default(),
            },
        );
        // A BABE digest item is needed to check authorship
        let digest = sp_runtime::DigestItem::PreRuntime(*b"BABE", digest_data.encode());
        <frame_system::Pallet<T>>::deposit_log(digest);
        populate_pool::<T>(i)?;

        #[extrinsic_call]
        _(
            RawOrigin::None,
            ComputationResult {
                distances: vec![Perbill::one(); i as usize],
            },
        );

        Ok(())
    }

    #[benchmark]
    fn force_update_evaluation(
        i: Linear<1, MAX_EVALUATIONS_PER_SESSION>,
    ) -> Result<(), BenchmarkError> {
        let idty = T::IdtyIndex::one();
        let caller: T::AccountId = pallet_identity::Identities::<T>::get(idty)
            .unwrap()
            .owner_key;
        populate_pool::<T>(i)?;

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            caller,
            ComputationResult {
                distances: vec![Perbill::one(); i as usize],
            },
        );

        Ok(())
    }

    #[benchmark]
    fn force_valid_distance_status() {
        let idty = T::IdtyIndex::one();

        #[extrinsic_call]
        _(RawOrigin::Root, idty);

        assert_has_event::<T>(Event::<T>::EvaluatedValid { idty_index: idty }.into());
    }

    #[benchmark]
    fn on_finalize() {
        DidUpdate::<T>::set(true);

        #[block]
        {
            Pallet::<T>::on_finalize(Default::default());
        }

        assert!(!DidUpdate::<T>::get());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
