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

//! # Distance Pallet
//!
//! The distance pallet utilizes results provided in a file by the `distance-oracle` offchain worker.
//! At a some point, an inherent is called to submit the results of this file to the blockchain.
//! The pallet then selects the median of the results (reach perbill) from an evaluation pool and fills the storage with the result status.
//! The status of an identity can be:
//!
//! - **Non-existent**: Distance evaluation has not been requested or has expired.
//! - **Pending**: Distance evaluation for this identity has been requested and is awaiting results after two evaluation periods.
//! - **Valid**: Distance has been evaluated positively for this identity.
//!
//! The evaluation result is used by the `duniter-wot` pallet to determine if an identity can gain or should lose membership in the web of trust.
//!
//! ## Process
//!
//! Any account can request a distance evaluation for a given identity provided it has enough currency to reserve. In this case, the distance status is marked as pending, and in the next evaluation period, inherents can start to publish results.
//!
//! This is the process for publishing a result:
//!
//! 1. A local worker creates a file containing the computation result.
//! 2. An inherent is created with the data from this file.
//! 3. The author is registered as an evaluator.
//! 4. The result is added to the current evaluation pool.
//! 5. A flag is set to prevent other distance evaluations in the same block.
//!
//! At the start of each new evaluation period:
//!
//! 1. Old results set to expire at this period are removed.
//! 2. Results from the current pool (results from the previous period's pool) are processed, and for each identity:
//!     - The median of the distance results for this identity is chosen.
//!     - If the distance is acceptable, it is marked as valid.
//!     - If the distance is not acceptable, the result for this identity is discarded, and reserved currency is slashed (from the account which requested the evaluation).
//!
//! Then, in other pallets, when a membership is claimed, it is possible to check if there is a valid distance evaluation for this identity.
//!
//! ## Pools
//!
//! Evaluation pools consist of two components:
//!
//! - A set of evaluators.
//! - A vector of results.
//!
//! The evaluations are divided into three pools:
//!
//! - Pool number N - 1 % 3: Results from the previous evaluation period used in the current one (emptied for the next evaluation period).
//! - Pool number N + 0 % 3: Inherent results are added here.
//! - Pool number N + 1 % 3: Identities are added here for evaluation.

#![cfg_attr(not(feature = "std"), no_std)]

mod median;
pub mod traits;
mod types;
mod weights;

pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
pub use traits::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::{
    traits::{
        fungible::{self, hold, Credit, Mutate, MutateHold},
        tokens::Precision,
        OnUnbalanced, StorageVersion,
    },
    DefaultNoBound,
};
use sp_distance::{InherentError, INHERENT_IDENTIFIER};
use sp_inherents::{InherentData, InherentIdentifier};
use sp_runtime::{
    traits::{One, Zero},
    Saturating,
};

type IdtyIndex = u32;

/// Maximum number of identities to be evaluated in an evaluation period.
pub const MAX_EVALUATIONS_PER_SESSION: u32 = 1_300; // See https://git.duniter.org/nodes/rust/duniter-v2s/-/merge_requests/252
/// Maximum number of evaluators in an evaluation period.
pub const MAX_EVALUATORS_PER_SESSION: u32 = 100;

#[frame_support::pallet()]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::Perbill;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type BalanceOf<T> = <<T as Config>::Currency as fungible::Inspect<AccountIdOf<T>>>::Balance;

    #[pallet::composite_enum]
    pub enum HoldReason {
        /// The funds are held as deposit for the distance evaluation.
        DistanceHold,
    }

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
    {
        /// Currency type used in this pallet for reserve and slash operations.
        type Currency: Mutate<Self::AccountId>
            + MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>
            + hold::Balanced<Self::AccountId>;

        /// The overarching hold reason type.
        type RuntimeHoldReason: From<HoldReason>;

        /// The amount reserved during evaluation.
        #[pallet::constant]
        type EvaluationPrice: Get<BalanceOf<Self>>;

        /// The evaluation period in blocks.
        /// Since the evaluation uses 3 pools, the total evaluation time will be 3 * EvaluationPeriod.
        #[pallet::constant]
        type EvaluationPeriod: Get<u32>;

        /// The maximum distance used to define a referee's accessibility.
        /// This value is not used by the runtime but is needed by the client distance oracle.
        #[pallet::constant]
        type MaxRefereeDistance: Get<u32>;

        /// The minimum ratio of accessible referees required.
        #[pallet::constant]
        type MinAccessibleReferees: Get<Perbill>;

        /// Handler for unbalanced reduction when invalid distance causes a slash.
        type OnUnbalanced: OnUnbalanced<Credit<Self::AccountId, Self::Currency>>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;

        /// A handler that is called when a distance evaluation is successfully validated.
        type OnValidDistanceStatus: OnValidDistanceStatus<Self>;

        /// A trait that provides a method to check if a distance evaluation request is allowed.
        type CheckRequestDistanceEvaluation: CheckRequestDistanceEvaluation<Self>;
    }

    // STORAGE //

    /// The first evaluation pool for distance evaluation queuing identities to evaluate for a given
    /// evaluator account.
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

    /// The second evaluation pool for distance evaluation queuing identities to evaluate for a given
    /// evaluator account.
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

    /// The third evaluation pool for distance evaluation queuing identities to evaluate for a given
    /// evaluator account.
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

    /// The block at which the distance is evaluated.
    #[pallet::storage]
    pub type EvaluationBlock<T: Config> =
        StorageValue<_, <T as frame_system::Config>::Hash, ValueQuery>;

    /// The pending evaluation requesters.
    #[pallet::storage]
    #[pallet::getter(fn pending_evaluation_request)]
    pub type PendingEvaluationRequest<T: Config> = StorageMap<
        _,
        Twox64Concat,
        <T as pallet_identity::Config>::IdtyIndex,
        <T as frame_system::Config>::AccountId,
        OptionQuery,
    >;

    /// Store if the evaluation was updated in this block.
    #[pallet::storage]
    pub(super) type DidUpdate<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// The current evaluation pool index.
    #[pallet::storage]
    #[pallet::getter(fn current_pool_index)]
    pub(super) type CurrentPoolIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A distance evaluation was requested.
        EvaluationRequested {
            idty_index: T::IdtyIndex,
            who: T::AccountId,
        },
        /// Distance rule was found valid.
        EvaluatedValid {
            idty_index: T::IdtyIndex,
            distance: Perbill,
        },
        /// Distance rule was found invalid.
        EvaluatedInvalid {
            idty_index: T::IdtyIndex,
            distance: Perbill,
        },
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

    #[pallet::genesis_config]
    #[derive(DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub _config: core::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            CurrentPoolIndex::<T>::put(0u32);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(block: BlockNumberFor<T>) -> Weight
        where
            BlockNumberFor<T>: From<u32>,
        {
            let mut weight = <T as pallet::Config>::WeightInfo::on_initialize_overhead();
            if block % BlockNumberFor::<T>::one().saturating_mul(T::EvaluationPeriod::get().into())
                == BlockNumberFor::<T>::zero()
            {
                let index = (CurrentPoolIndex::<T>::get() + 1) % 3;
                CurrentPoolIndex::<T>::put(index);
                weight = weight
                    .saturating_add(Self::do_evaluation(index))
                    .saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }
            weight.saturating_add(<T as pallet::Config>::WeightInfo::on_finalize())
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            DidUpdate::<T>::take();
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request evaluation of the caller's identity distance.
        ///
        /// This function allows the caller to request an evaluation of their distance.
        /// A positive evaluation will lead to claiming or renewing membership, while a negative
        /// evaluation will result in slashing for the caller.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::request_distance_evaluation())]
        pub fn request_distance_evaluation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let idty = Self::check_request_distance_evaluation_self(&who)?;

            Pallet::<T>::do_request_distance_evaluation(&who, idty)?;
            Ok(().into())
        }

        /// Request evaluation of a target identity's distance.
        ///
        /// This function allows the caller to request an evaluation of a specific target identity's distance.
        /// This action is only permitted for unvalidated identities.
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

        /// Push an evaluation result to the pool.
        ///
        /// This inherent function is called internally by validators to push an evaluation result
        /// to the evaluation pool.
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

        /// Force push an evaluation result to the pool.
        ///
        /// It is primarily used for testing purposes.
        ///
        /// - `origin`: Must be `Root`.
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

        /// Force set the distance evaluation status of an identity.
        ///
        /// It is primarily used for testing purposes.
        ///
        /// - `origin`: Must be `Root`.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_valid_distance_status())]
        pub fn force_valid_distance_status(
            origin: OriginFor<T>,
            identity: <T as pallet_identity::Config>::IdtyIndex,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::do_valid_distance_status(identity, Perbill::one());
            Ok(())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Mutate the evaluation pool containing:
        /// * when this period begins: the evaluation results to be applied.
        /// * when this period ends: the evaluation requests.
        fn mutate_current_pool<
            R,
            F: FnOnce(
                &mut EvaluationPool<
                    <T as frame_system::Config>::AccountId,
                    <T as pallet_identity::Config>::IdtyIndex,
                >,
            ) -> R,
        >(
            index: u32,
            f: F,
        ) -> R {
            match index {
                0 => EvaluationPool2::<T>::mutate(f),
                1 => EvaluationPool0::<T>::mutate(f),
                2 => EvaluationPool1::<T>::mutate(f),
                _ => unreachable!("index < 3"),
            }
        }

        /// Mutate the evaluation pool containing the results sent by evaluators for this period.
        fn mutate_next_pool<
            R,
            F: FnOnce(
                &mut EvaluationPool<
                    <T as frame_system::Config>::AccountId,
                    <T as pallet_identity::Config>::IdtyIndex,
                >,
            ) -> R,
        >(
            index: u32,
            f: F,
        ) -> R {
            match index {
                0 => EvaluationPool0::<T>::mutate(f),
                1 => EvaluationPool1::<T>::mutate(f),
                2 => EvaluationPool2::<T>::mutate(f),
                _ => unreachable!("index < 3"),
            }
        }

        /// Take (*and leave empty*) the evaluation pool containing:
        /// * when this period begins: the evaluation results to be applied.
        /// * when this period ends: the evaluation requests.
        #[allow(clippy::type_complexity)]
        fn take_current_pool(
            index: u32,
        ) -> EvaluationPool<
            <T as frame_system::Config>::AccountId,
            <T as pallet_identity::Config>::IdtyIndex,
        > {
            match index {
                0 => EvaluationPool2::<T>::take(),
                1 => EvaluationPool0::<T>::take(),
                2 => EvaluationPool1::<T>::take(),
                _ => unreachable!("index % 3 < 3"),
            }
        }

        /// Check if requested distance evaluation is allowed.
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

        /// Request distance evaluation in the current pool.
        fn do_request_distance_evaluation(
            who: &T::AccountId,
            idty_index: <T as pallet_identity::Config>::IdtyIndex,
        ) -> Result<(), DispatchError> {
            Pallet::<T>::mutate_current_pool(CurrentPoolIndex::<T>::get(), |current_pool| {
                // extrinsics are transactional by default, this check might not be needed
                ensure!(
                    current_pool.evaluations.len() < (MAX_EVALUATIONS_PER_SESSION as usize),
                    Error::<T>::QueueFull
                );

                T::Currency::hold(
                    &HoldReason::DistanceHold.into(),
                    who,
                    <T as Config>::EvaluationPrice::get(),
                )?;

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
            })
        }

        /// Update distance evaluation in the next pool.
        fn do_update_evaluation(
            evaluator: <T as frame_system::Config>::AccountId,
            computation_result: ComputationResult,
        ) -> DispatchResult {
            Pallet::<T>::mutate_next_pool(CurrentPoolIndex::<T>::get(), |result_pool| {
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

        /// Set the distance status using for an identity.
        pub fn do_valid_distance_status(
            idty: <T as pallet_identity::Config>::IdtyIndex,
            distance: Perbill,
        ) {
            // callback
            T::OnValidDistanceStatus::on_valid_distance_status(idty);
            // deposit event
            Self::deposit_event(Event::EvaluatedValid {
                idty_index: idty,
                distance,
            });
        }

        /// Perform evaluation for a specified pool.
        ///
        /// This function executes evaluation logic based on the provided pool index. It retrieves the current
        /// evaluation pool for the index, processes each evaluation, and handles the outcomes based on the
        /// computed median distances. If a positive evaluation result is obtained, it releases reserved funds
        /// and updates the distance status accordingly. For negative or inconclusive results, it slashes funds
        /// or releases them, respectively.
        pub fn do_evaluation(index: u32) -> Weight {
            let mut weight = <T as pallet::Config>::WeightInfo::do_evaluation_overhead();
            // set evaluation block
            EvaluationBlock::<T>::set(frame_system::Pallet::<T>::parent_hash());

            // Apply the results from the current pool (which was previous period's result pool)
            // We take the results so the pool is left empty for the new period.
            #[allow(clippy::type_complexity)]
            let current_pool: EvaluationPool<
                <T as frame_system::Config>::AccountId,
                <T as pallet_identity::Config>::IdtyIndex,
            > = Pallet::<T>::take_current_pool(index);

            for (idty, median_acc) in current_pool.evaluations.into_iter() {
                let mut distance_result: Option<Perbill> = None;
                // Retrieve the result of the computation from the median accumulator
                if let Some(median_result) = median_acc.get_median() {
                    let distance = match median_result {
                        MedianResult::One(m) => m,
                        MedianResult::Two(m1, m2) => m1 + (m2 - m1) / 2, // Avoid overflow (since max is 1)
                    };
                    // Update distance result
                    distance_result = Some(distance);
                }

                // If there's a pending evaluation request with the provided identity
                if let Some(requester) = PendingEvaluationRequest::<T>::take(idty) {
                    // If distance_result is available
                    if let Some(distance) = distance_result {
                        if distance >= T::MinAccessibleReferees::get() {
                            // Positive result, unreserve and apply
                            let _ = T::Currency::release(
                                &HoldReason::DistanceHold.into(),
                                &requester,
                                <T as Config>::EvaluationPrice::get(),
                                Precision::Exact,
                            );
                            Self::do_valid_distance_status(idty, distance);
                            weight = weight.saturating_add(
                                <T as pallet::Config>::WeightInfo::do_evaluation_success()
                                    .saturating_sub(
                                        <T as pallet::Config>::WeightInfo::do_evaluation_overhead(),
                                    ),
                            );
                        } else {
                            // Negative result, slash and deposit event
                            let (imbalance, _) = <T::Currency as hold::Balanced<_>>::slash(
                                &HoldReason::DistanceHold.into(),
                                &requester,
                                <T as Config>::EvaluationPrice::get(),
                            );
                            T::OnUnbalanced::on_unbalanced(imbalance);
                            Self::deposit_event(Event::EvaluatedInvalid {
                                idty_index: idty,
                                distance,
                            });
                            weight = weight.saturating_add(
                                <T as pallet::Config>::WeightInfo::do_evaluation_failure()
                                    .saturating_sub(
                                        <T as pallet::Config>::WeightInfo::do_evaluation_overhead(),
                                    ),
                            );
                        }
                    } else {
                        // No result, unreserve
                        let _ = T::Currency::release(
                            &HoldReason::DistanceHold.into(),
                            &requester,
                            <T as Config>::EvaluationPrice::get(),
                            Precision::Exact,
                        );
                        weight = weight.saturating_add(
                            <T as pallet::Config>::WeightInfo::do_evaluation_failure()
                                .saturating_sub(
                                    <T as pallet::Config>::WeightInfo::do_evaluation_overhead(),
                                ),
                        );
                    }
                }
                // If evaluation happened without request, it's ok to do nothing
            }
            weight
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
