// Copyright 2022 Axiom-Team
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

#![cfg_attr(not(feature = "std"), no_std)]

mod median;
mod traits;
mod types;
mod weights;

pub use pallet::*;
pub use traits::*;
pub use types::*;
// pub use weights::WeightInfo;

use frame_support::traits::StorageVersion;
use pallet_authority_members::SessionIndex;
use sp_distance::{InherentError, INHERENT_IDENTIFIER};
use sp_inherents::{InherentData, InherentIdentifier};
use sp_std::convert::TryInto;

type IdtyIndex = u32;

/// Maximum number of identities to be evaluated in a session
pub const MAX_EVALUATIONS_PER_SESSION: u32 = 600;
/// Maximum number of evaluators in a session
pub const MAX_EVALUATORS_PER_SESSION: u32 = 100;

#[frame_support::pallet(dev_mode)] // dev mode while waiting for benchmarks
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::Perbill;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    // #[pallet::generate_store(pub(super) trait Store)] // deprecated
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);
    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_authorship::Config
        + pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_session::Config
    {
        type Currency: ReservableCurrency<Self::AccountId>;
        /// Amount reserved during evaluation
        #[pallet::constant]
        type EvaluationPrice: Get<
            <Self::Currency as frame_support::traits::Currency<Self::AccountId>>::Balance,
        >;
        /// Minimum ratio of accessible referees
        #[pallet::constant]
        type MinAccessibleReferees: Get<Perbill>;
        /// Number of session to keep a positive evaluation result
        type ResultExpiration: Get<u32>;
        // /// Type representing the weight of this pallet
        // type WeightInfo: WeightInfo;
    }

    // STORAGE //

    /// Identities queued for distance evaluation
    #[pallet::storage]
    #[pallet::getter(fn evaluation_pool_0)]
    pub type EvaluationPool0<T: Config> = StorageValue<
        _,
        EvaluationPool<
            <T as frame_system::Config>::AccountId,
            <T as pallet_identity::Config>::IdtyIndex,
        >,
        ValueQuery,
    >;
    /// Identities queued for distance evaluation
    #[pallet::storage]
    #[pallet::getter(fn evaluation_pool_1)]
    pub type EvaluationPool1<T: Config> = StorageValue<
        _,
        EvaluationPool<
            <T as frame_system::Config>::AccountId,
            <T as pallet_identity::Config>::IdtyIndex,
        >,
        ValueQuery,
    >;
    /// Identities queued for distance evaluation
    #[pallet::storage]
    #[pallet::getter(fn evaluation_pool_2)]
    pub type EvaluationPool2<T: Config> = StorageValue<
        _,
        EvaluationPool<
            <T as frame_system::Config>::AccountId,
            <T as pallet_identity::Config>::IdtyIndex,
        >,
        ValueQuery,
    >;

    /// Block for which the distance rule must be checked
    #[pallet::storage]
    pub type EvaluationBlock<T: Config> =
        StorageValue<_, <T as frame_system::Config>::Hash, ValueQuery>;

    /// Distance evaluation status by identity
    ///
    /// * `.0` is the account who requested an evaluation and reserved the price,
    ///   for whom the price will be unreserved or slashed when the evaluation completes.
    /// * `.1` is the status of the evaluation.
    #[pallet::storage]
    #[pallet::getter(fn identity_distance_status)]
    pub type IdentityDistanceStatus<T: Config> = StorageMap<
        _,
        Twox64Concat,
        <T as pallet_identity::Config>::IdtyIndex,
        (<T as frame_system::Config>::AccountId, DistanceStatus),
        OptionQuery,
    >;

    /// Identities by distance status expiration session index
    #[pallet::storage]
    #[pallet::getter(fn distance_status_expire_on)]
    pub type DistanceStatusExpireOn<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u32,
        BoundedVec<
            <T as pallet_identity::Config>::IdtyIndex,
            ConstU32<MAX_EVALUATIONS_PER_SESSION>,
        >,
        ValueQuery,
    >;

    /// Did evaluation get updated in this block?
    #[pallet::storage]
    pub(super) type DidUpdate<T: Config> = StorageValue<_, bool, ValueQuery>;

    // session_index % 3:
    //   storage_id + 0 => pending
    //   storage_id + 1 => receives results
    //   storage_id + 2 => receives new identities
    // (this avoids problems for session_index < 3)

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        AlreadyInEvaluation,
        CannotReserve,
        ManyEvaluationsByAuthor,
        ManyEvaluationsInBlock,
        NoAuthor,
        NoIdentity,
        NonEligibleForEvaluation,
        QueueFull,
        TooManyEvaluators,
        WrongResultLength,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// dummy `on_initialize` to return the weight used in `on_finalize`.
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // weight of `on_finalize`
            //T::WeightInfo::on_finalize()// TODO uncomment when benchmarking
            Weight::zero()
        }

        /// # <weight>
        /// - `O(1)`
        /// - 1 storage deletion (codec `O(1)`).
        /// # </weight>
        fn on_finalize(_n: BlockNumberFor<T>) {
            DidUpdate::<T>::take();
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request an identity to be evaluated
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        // #[pallet::weight(T::WeightInfo::request_distance_evaluation())]
        pub fn request_distance_evaluation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let idty =
                pallet_identity::IdentityIndexOf::<T>::get(&who).ok_or(Error::<T>::NoIdentity)?;

            ensure!(
                IdentityDistanceStatus::<T>::get(idty).is_none(),
                Error::<T>::AlreadyInEvaluation
            );

            Pallet::<T>::do_request_distance_evaluation(who, idty)?;
            Ok(().into())
        }

        /// (Inherent) Push an evaluation result to the pool
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        // #[pallet::weight(T::WeightInfo::update_evaluation())]
        pub fn update_evaluation(
            origin: OriginFor<T>,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            ensure_none(origin)?;
            ensure!(
                !DidUpdate::<T>::exists(),
                Error::<T>::ManyEvaluationsInBlock,
            );
            let author = pallet_authorship::Pallet::<T>::author().ok_or(Error::<T>::NoAuthor)?;

            Pallet::<T>::do_update_evaluation(author, computation_result)?;

            DidUpdate::<T>::set(true);
            Ok(())
        }

        /// Push an evaluation result to the pool
        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        // #[pallet::weight(T::WeightInfo::force_update_evaluation())]
        pub fn force_update_evaluation(
            origin: OriginFor<T>,
            evaluator: <T as frame_system::Config>::AccountId,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Pallet::<T>::do_update_evaluation(evaluator, computation_result)
        }

        /// Set the distance evaluation status of an identity
        ///
        /// Removes the status if `status` is `None`.
        ///
        /// * `status.0` is the account for whom the price will be unreserved or slashed
        ///   when the evaluation completes.
        /// * `status.1` is the status of the evaluation.
        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        // #[pallet::weight(T::WeightInfo::force_set_distance_status())]
        pub fn force_set_distance_status(
            origin: OriginFor<T>,
            identity: <T as pallet_identity::Config>::IdtyIndex,
            status: Option<(<T as frame_system::Config>::AccountId, DistanceStatus)>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            IdentityDistanceStatus::<T>::set(identity, status);
            DistanceStatusExpireOn::<T>::mutate(
                pallet_session::CurrentIndex::<T>::get() + T::ResultExpiration::get(),
                move |identities| {
                    identities
                        .try_push(identity)
                        .map_err(|_| Error::<T>::ManyEvaluationsInBlock.into())
                },
            )
        }
    }

    // BENCHMARK FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Force the distance status using IdtyIndex and AccountId
        /// only to prepare identity for benchmarking.
        pub fn set_distance_status(
            identity: <T as pallet_identity::Config>::IdtyIndex,
            status: Option<(<T as frame_system::Config>::AccountId, DistanceStatus)>,
        ) -> DispatchResult {
            IdentityDistanceStatus::<T>::set(identity, status);
            DistanceStatusExpireOn::<T>::mutate(
                pallet_session::CurrentIndex::<T>::get() + T::ResultExpiration::get(),
                move |identities| {
                    identities
                        .try_push(identity)
                        .map_err(|_| Error::<T>::ManyEvaluationsInBlock.into())
                },
            )
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Mutate the evaluation pool containing:
        /// * when this session begins: the evaluation results to be applied
        /// * when this session ends: the evaluation requests
        fn mutate_current_pool<
            R,
            F: FnOnce(
                &mut EvaluationPool<
                    <T as frame_system::Config>::AccountId,
                    <T as pallet_identity::Config>::IdtyIndex,
                >,
            ) -> R,
        >(
            index: SessionIndex,
            f: F,
        ) -> R {
            match index % 3 {
                0 => EvaluationPool2::<T>::mutate(f),
                1 => EvaluationPool0::<T>::mutate(f),
                2 => EvaluationPool1::<T>::mutate(f),
                _ => unreachable!("index % 3 < 3"),
            }
        }
        /// Mutate the evaluation pool containing the results sent by evaluators on this session.
        fn mutate_next_pool<
            R,
            F: FnOnce(
                &mut EvaluationPool<
                    <T as frame_system::Config>::AccountId,
                    <T as pallet_identity::Config>::IdtyIndex,
                >,
            ) -> R,
        >(
            index: SessionIndex,
            f: F,
        ) -> R {
            match index % 3 {
                0 => EvaluationPool0::<T>::mutate(f),
                1 => EvaluationPool1::<T>::mutate(f),
                2 => EvaluationPool2::<T>::mutate(f),
                _ => unreachable!("index % 3 < 3"),
            }
        }

        /// Take (and leave empty) the evaluation pool containing:
        /// * when this session begins: the evaluation results to be applied
        /// * when this session ends: the evaluation requests
        #[allow(clippy::type_complexity)]
        fn take_current_pool(
            index: SessionIndex,
        ) -> EvaluationPool<
            <T as frame_system::Config>::AccountId,
            <T as pallet_identity::Config>::IdtyIndex,
        > {
            match index % 3 {
                0 => EvaluationPool2::<T>::take(),
                1 => EvaluationPool0::<T>::take(),
                2 => EvaluationPool1::<T>::take(),
                _ => unreachable!("index % 3 < 3"),
            }
        }

        fn do_request_distance_evaluation(
            who: T::AccountId,
            idty_index: <T as pallet_identity::Config>::IdtyIndex,
        ) -> Result<(), DispatchError> {
            Pallet::<T>::mutate_current_pool(
                pallet_session::CurrentIndex::<T>::get(),
                |current_pool| {
                    ensure!(
                        current_pool.evaluations.len() < (MAX_EVALUATIONS_PER_SESSION as usize),
                        Error::<T>::QueueFull
                    );

                    T::Currency::reserve(&who, <T as Config>::EvaluationPrice::get())?;

                    current_pool
                        .evaluations
                        .try_push((idty_index, median::MedianAcc::new()))
                        .map_err(|_| Error::<T>::QueueFull)?;

                    IdentityDistanceStatus::<T>::insert(idty_index, (who, DistanceStatus::Pending));

                    DistanceStatusExpireOn::<T>::mutate(
                        pallet_session::CurrentIndex::<T>::get() + T::ResultExpiration::get(),
                        move |identities| identities.try_push(idty_index).ok(),
                    );

                    Ok(())
                },
            )
        }

        fn do_update_evaluation(
            evaluator: <T as frame_system::Config>::AccountId,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            Pallet::<T>::mutate_next_pool(pallet_session::CurrentIndex::<T>::get(), |result_pool| {
                ensure!(
                    computation_result.distances.len() == result_pool.evaluations.len(),
                    Error::<T>::WrongResultLength
                );

                if result_pool
                    .evaluators
                    .try_insert(evaluator)
                    .map_err(|_| Error::<T>::TooManyEvaluators)?
                {
                    for (distance_value, (_identity, median_acc)) in computation_result
                        .distances
                        .into_iter()
                        .zip(result_pool.evaluations.iter_mut())
                    {
                        median_acc.push(distance_value);
                    }

                    Ok(())
                } else {
                    Err(Error::<T>::ManyEvaluationsByAuthor.into())
                }
            })
        }
    }

    impl<T: Config> pallet_authority_members::OnNewSession for Pallet<T> {
        fn on_new_session(index: SessionIndex) -> Weight {
            EvaluationBlock::<T>::set(frame_system::Pallet::<T>::parent_hash());

            // Make results expire
            DistanceStatusExpireOn::<T>::remove(index);

            // Apply the results from the current pool (which was previous session's result pool)
            // We take the results so the pool is left empty for the new session.
            #[allow(clippy::type_complexity)]
            let current_pool: EvaluationPool<
                <T as frame_system::Config>::AccountId,
                <T as pallet_identity::Config>::IdtyIndex,
            > = Pallet::<T>::take_current_pool(index);
            for (idty, median_acc) in current_pool.evaluations.into_iter() {
                if let Some(median_result) = median_acc.get_median() {
                    let median = match median_result {
                        MedianResult::One(m) => m,
                        MedianResult::Two(m1, m2) => m1 + (m2 - m1) / 2, // Avoid overflow (since max is 1)
                    };
                    if median >= T::MinAccessibleReferees::get() {
                        IdentityDistanceStatus::<T>::mutate(idty, |entry| {
                            entry.as_mut().map(|(account_id, status)| {
                                T::Currency::unreserve(
                                    account_id,
                                    <T as Config>::EvaluationPrice::get(),
                                );
                                *status = DistanceStatus::Valid;
                            })
                        });
                    } else if let Some((account_id, _status)) =
                        IdentityDistanceStatus::<T>::take(idty)
                    {
                        <T as Config>::Currency::slash_reserved(
                            &account_id,
                            <T as Config>::EvaluationPrice::get(),
                        );
                    }
                } else if let Some((account_id, _status)) = IdentityDistanceStatus::<T>::take(idty)
                {
                    <T as Config>::Currency::unreserve(
                        &account_id,
                        <T as Config>::EvaluationPrice::get(),
                    );
                }
            }
            Weight::zero()
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = InherentError;

        const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

        fn create_inherent(data: &InherentData) -> Option<Self::Call> {
            data.get_data::<ComputationResult>(&INHERENT_IDENTIFIER)
                .expect("Distance inherent data not correctly encoded")
                .map(|inherent_data| Call::update_evaluation {
                    computation_result: inherent_data,
                })
        }
        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Self::Call::update_evaluation { .. })
        }
    }
}
