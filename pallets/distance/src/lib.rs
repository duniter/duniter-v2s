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
pub mod traits;
mod types;
mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
pub use traits::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::traits::StorageVersion;
use pallet_authority_members::SessionIndex;
use sp_distance::{InherentError, INHERENT_IDENTIFIER};
use sp_inherents::{InherentData, InherentIdentifier};
use sp_std::convert::TryInto;
use sp_std::prelude::*;

type IdtyIndex = u32;

/// Maximum number of identities to be evaluated in a session
pub const MAX_EVALUATIONS_PER_SESSION: u32 = 600;
/// Maximum number of evaluators in a session
pub const MAX_EVALUATORS_PER_SESSION: u32 = 100;

#[frame_support::pallet()]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::Perbill;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
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
        /// Currency type used in this pallet (used for reserve/slash)
        type Currency: ReservableCurrency<Self::AccountId>;
        /// Amount reserved during evaluation
        #[pallet::constant]
        type EvaluationPrice: Get<
            <Self::Currency as frame_support::traits::Currency<Self::AccountId>>::Balance,
        >;
        /// Maximum distance used to define referee's accessibility
        /// Unused by runtime but needed by client distance oracle
        #[pallet::constant]
        type MaxRefereeDistance: Get<u32>;
        /// Minimum ratio of accessible referees
        #[pallet::constant]
        type MinAccessibleReferees: Get<Perbill>;
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
        /// Handler for successful distance evaluation
        type OnValidDistanceStatus: OnValidDistanceStatus<Self>;
        /// Trait to check that distance evaluation request is allowed
        type CheckRequestDistanceEvaluation: CheckRequestDistanceEvaluation<Self>;
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

    /// Pending evaluation requesters
    ///
    /// account who requested an evaluation and reserved the price,
    ///   for whom the price will be unreserved or slashed when the evaluation completes.
    #[pallet::storage]
    #[pallet::getter(fn pending_evaluation_request)]
    pub type PendingEvaluationRequest<T: Config> = StorageMap<
        _,
        Twox64Concat,
        <T as pallet_identity::Config>::IdtyIndex,
        <T as frame_system::Config>::AccountId,
        OptionQuery,
    >;

    /// Did evaluation get updated in this block?
    #[pallet::storage]
    pub(super) type DidUpdate<T: Config> = StorageValue<_, bool, ValueQuery>;

    // session_index % 3:
    //   storage_id + 0 => pending
    //   storage_id + 1 => receives results
    //   storage_id + 2 => receives new identities
    // (this avoids problems for session_index < 3)
    //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A distance evaluation was requested.
        EvaluationRequested {
            idty_index: T::IdtyIndex,
            who: T::AccountId,
        },
        /// Distance rule was found valid.
        EvaluatedValid { idty_index: T::IdtyIndex },
        /// Distance rule was found invalid.
        EvaluatedInvalid { idty_index: T::IdtyIndex },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Distance is already under evaluation.
        AlreadyInEvaluation,
        /// Too many evaluations requested by author.
        TooManyEvaluationsByAuthor,
        /// Too many evaluations for this block.
        TooManyEvaluationsInBlock,
        /// No author for this block.
        NoAuthor,
        /// Caller has no identity.
        CallerHasNoIdentity,
        /// Caller identity not found.
        CallerIdentityNotFound,
        /// Caller not member.
        CallerNotMember,
        // Caller status can only be Unvalidated, Member or NotMember.
        CallerStatusInvalid,
        /// Target identity not found.
        TargetIdentityNotFound,
        /// Evaluation queue is full.
        QueueFull,
        /// Too many evaluators in the current evaluation pool.
        TooManyEvaluators,
        /// Evaluation result has a wrong length.
        WrongResultLength,
        /// Targeted distance evaluation request is only possible for an unvalidated identity.
        TargetMustBeUnvalidated,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// dummy `on_initialize` to return the weight used in `on_finalize`.
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // weight of `on_finalize`
            <T as pallet::Config>::WeightInfo::on_finalize()
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
        /// Request caller identity to be evaluated
        /// positive evaluation will result in claim/renew membership
        /// negative evaluation will result in slash for caller
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::request_distance_evaluation())]
        pub fn request_distance_evaluation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let idty = Self::check_request_distance_evaluation_self(&who)?;

            Pallet::<T>::do_request_distance_evaluation(&who, idty)?;
            Ok(().into())
        }

        /// Request target identity to be evaluated
        /// only possible for unvalidated identity
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::request_distance_evaluation_for())]
        pub fn request_distance_evaluation_for(
            origin: OriginFor<T>,
            target: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::check_request_distance_evaluation_for(&who, target)?;

            Pallet::<T>::do_request_distance_evaluation(&who, target)?;
            Ok(().into())
        }

        /// (Inherent) Push an evaluation result to the pool
        /// this is called internally by validators (= inherent)
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_evaluation(MAX_EVALUATIONS_PER_SESSION))]
        pub fn update_evaluation(
            origin: OriginFor<T>,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            // no origin = inherent
            ensure_none(origin)?;
            ensure!(
                !DidUpdate::<T>::exists(),
                Error::<T>::TooManyEvaluationsInBlock,
            );
            let author = pallet_authorship::Pallet::<T>::author().ok_or(Error::<T>::NoAuthor)?;

            Pallet::<T>::do_update_evaluation(author, computation_result)?;

            DidUpdate::<T>::set(true);
            Ok(())
        }

        /// Force push an evaluation result to the pool
        // (it is convenient to have this call in end2end tests)
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_update_evaluation(MAX_EVALUATIONS_PER_SESSION))]
        pub fn force_update_evaluation(
            origin: OriginFor<T>,
            evaluator: <T as frame_system::Config>::AccountId,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Pallet::<T>::do_update_evaluation(evaluator, computation_result)
        }

        /// Force set the distance evaluation status of an identity
        // (it is convenient to have this in test network)
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_valid_distance_status())]
        pub fn force_valid_distance_status(
            origin: OriginFor<T>,
            identity: <T as pallet_identity::Config>::IdtyIndex,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::do_valid_distance_status(identity);
            Ok(())
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

        /// check that request distance evaluation is allowed
        fn check_request_distance_evaluation_self(
            who: &T::AccountId,
        ) -> Result<<T as pallet_identity::Config>::IdtyIndex, DispatchError> {
            // caller has an identity
            let idty_index = pallet_identity::IdentityIndexOf::<T>::get(who)
                .ok_or(Error::<T>::CallerHasNoIdentity)?;
            let idty = pallet_identity::Identities::<T>::get(idty_index)
                .ok_or(Error::<T>::CallerIdentityNotFound)?;
            // caller is (Unvalidated, Member, NotMember)
            ensure!(
                idty.status == pallet_identity::IdtyStatus::Unvalidated
                    || idty.status == pallet_identity::IdtyStatus::Member
                    || idty.status == pallet_identity::IdtyStatus::NotMember,
                Error::<T>::CallerStatusInvalid
            );
            Self::check_request_distance_evaluation_common(idty_index)?;
            Ok(idty_index)
        }

        /// check that targeted request distance evaluation is allowed
        fn check_request_distance_evaluation_for(
            who: &T::AccountId,
            target: <T as pallet_identity::Config>::IdtyIndex,
        ) -> Result<(), DispatchError> {
            // caller has an identity
            let caller_idty_index = pallet_identity::IdentityIndexOf::<T>::get(who)
                .ok_or(Error::<T>::CallerHasNoIdentity)?;
            let caller_idty = pallet_identity::Identities::<T>::get(caller_idty_index)
                .ok_or(Error::<T>::CallerIdentityNotFound)?;
            // caller is member
            ensure!(
                caller_idty.status == pallet_identity::IdtyStatus::Member,
                Error::<T>::CallerNotMember
            );
            // target has an identity
            let target_idty = pallet_identity::Identities::<T>::get(target)
                .ok_or(Error::<T>::TargetIdentityNotFound)?;
            // target is unvalidated
            ensure!(
                target_idty.status == pallet_identity::IdtyStatus::Unvalidated,
                Error::<T>::TargetMustBeUnvalidated
            );
            Self::check_request_distance_evaluation_common(target)?;
            Ok(())
        }

        // common checks between check_request_distance_evaluation _self and _for
        fn check_request_distance_evaluation_common(
            target: <T as pallet_identity::Config>::IdtyIndex,
        ) -> Result<(), DispatchError> {
            // no pending evaluation request
            ensure!(
                PendingEvaluationRequest::<T>::get(target).is_none(),
                Error::<T>::AlreadyInEvaluation
            );
            // external validation (wot)
            // - membership renewal antispam
            // - target has received enough certifications
            T::CheckRequestDistanceEvaluation::check_request_distance_evaluation(target)
        }

        /// request distance evaluation in current pool
        fn do_request_distance_evaluation(
            who: &T::AccountId,
            idty_index: <T as pallet_identity::Config>::IdtyIndex,
        ) -> Result<(), DispatchError> {
            Pallet::<T>::mutate_current_pool(
                pallet_session::CurrentIndex::<T>::get(),
                |current_pool| {
                    // extrinsics are transactional by default, this check might not be needed
                    ensure!(
                        current_pool.evaluations.len() < (MAX_EVALUATIONS_PER_SESSION as usize),
                        Error::<T>::QueueFull
                    );

                    T::Currency::reserve(who, <T as Config>::EvaluationPrice::get())?;

                    current_pool
                        .evaluations
                        .try_push((idty_index, median::MedianAcc::new()))
                        .map_err(|_| Error::<T>::QueueFull)?;

                    PendingEvaluationRequest::<T>::insert(idty_index, who);

                    Self::deposit_event(Event::EvaluationRequested {
                        idty_index,
                        who: who.clone(),
                    });
                    Ok(())
                },
            )
        }

        /// update distance evaluation in next pool
        fn do_update_evaluation(
            evaluator: <T as frame_system::Config>::AccountId,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            Pallet::<T>::mutate_next_pool(pallet_session::CurrentIndex::<T>::get(), |result_pool| {
                // evaluation must be provided for all identities (no more, no less)
                ensure!(
                    computation_result.distances.len() == result_pool.evaluations.len(),
                    Error::<T>::WrongResultLength
                );

                // insert the evaluator if not already there
                if result_pool
                    .evaluators
                    .try_insert(evaluator.clone())
                    .map_err(|_| Error::<T>::TooManyEvaluators)?
                {
                    // update the median accumulator with the new result
                    for (distance_value, (_identity, median_acc)) in computation_result
                        .distances
                        .into_iter()
                        .zip(result_pool.evaluations.iter_mut())
                    {
                        median_acc.push(distance_value);
                    }
                    Ok(())
                } else {
                    // one author can only submit one evaluation
                    Err(Error::<T>::TooManyEvaluationsByAuthor.into())
                }
            })
        }

        /// Set the distance status using IdtyIndex and AccountId
        pub fn do_valid_distance_status(idty: <T as pallet_identity::Config>::IdtyIndex) {
            // callback
            T::OnValidDistanceStatus::on_valid_distance_status(idty);
            // deposit event
            Self::deposit_event(Event::EvaluatedValid { idty_index: idty });
        }
    }

    impl<T: Config> pallet_authority_members::OnNewSession for Pallet<T> {
        fn on_new_session(index: SessionIndex) {
            // set evaluation block
            EvaluationBlock::<T>::set(frame_system::Pallet::<T>::parent_hash());

            // Apply the results from the current pool (which was previous session's result pool)
            // We take the results so the pool is left empty for the new session.
            #[allow(clippy::type_complexity)]
            let current_pool: EvaluationPool<
                <T as frame_system::Config>::AccountId,
                <T as pallet_identity::Config>::IdtyIndex,
            > = Pallet::<T>::take_current_pool(index);
            for (idty, median_acc) in current_pool.evaluations.into_iter() {
                // distance result
                let mut distance_result: Option<bool> = None;

                // get result of the computation
                if let Some(median_result) = median_acc.get_median() {
                    let median = match median_result {
                        MedianResult::One(m) => m,
                        MedianResult::Two(m1, m2) => m1 + (m2 - m1) / 2, // Avoid overflow (since max is 1)
                    };
                    // update distance result
                    distance_result = Some(median >= T::MinAccessibleReferees::get());
                }

                // take requester and perform unreserve or slash
                if let Some(requester) = PendingEvaluationRequest::<T>::take(idty) {
                    match distance_result {
                        None => {
                            // no result, unreserve
                            T::Currency::unreserve(
                                &requester,
                                <T as Config>::EvaluationPrice::get(),
                            );
                        }
                        Some(true) => {
                            // positive result, unreserve and apply
                            T::Currency::unreserve(
                                &requester,
                                <T as Config>::EvaluationPrice::get(),
                            );
                            Self::do_valid_distance_status(idty);
                        }
                        Some(false) => {
                            // negative result, slash and deposit event
                            T::Currency::slash_reserved(
                                &requester,
                                <T as Config>::EvaluationPrice::get(),
                            );
                            Self::deposit_event(Event::EvaluatedInvalid { idty_index: idty });
                        }
                    }
                }
                // if evaluation happened without request, it's ok to do nothing
            }
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
