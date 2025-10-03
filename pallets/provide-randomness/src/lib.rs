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

//! # Provides Randomness Pallet
//!
//! The Provides Randomness Pallet facilitates the generation of randomness within the Duniter blockchain.
//!
//! This pallet manages randomness requests and emits events upon requesting and fulfilling randomness.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::boxed_local)]

mod benchmarking;

mod types;
pub mod weights;

use frame_support::{
    pallet_prelude::Weight,
    traits::{
        fungible::{self, Balanced, Credit},
        tokens::{Fortitude, Precision, Preservation},
    },
};
use scale_info::prelude::vec::Vec;
use sp_core::H256;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

pub type RequestId = u64;

pub trait OnFilledRandomness {
    fn on_filled_randomness(request_id: RequestId, randomness: H256) -> Weight;
}
impl OnFilledRandomness for () {
    fn on_filled_randomness(_: RequestId, _: H256) -> Weight {
        Weight::zero()
    }
}

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{OnUnbalanced, Randomness, StorageVersion},
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H256;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type BalanceOf<T> = <<T as Config>::Currency as fungible::Inspect<AccountIdOf<T>>>::Balance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config<Hash = H256> {
        // The currency type.
        type Currency: fungible::Balanced<Self::AccountId> + fungible::Mutate<Self::AccountId>;

        /// Type providing the current epoch index.
        type GetCurrentEpochIndex: Get<u64>;

        /// Maximum number of not yet filled requests.
        #[pallet::constant]
        type MaxRequests: Get<u32>;

        /// The price of a request.
        #[pallet::constant]
        type RequestPrice: Get<BalanceOf<Self>>;

        /// Handler called when randomness is filled.
        type OnFilledRandomness: OnFilledRandomness;

        /// Handler for unbalanced reduction when the requestor pays fees.
        type OnUnbalanced: OnUnbalanced<Credit<Self::AccountId, Self::Currency>>;

        /// A safe source of randomness from the parent block.
        type ParentBlockRandomness: Randomness<Option<H256>, BlockNumberFor<Self>>;

        /// A safe source of randomness from one epoch ago.
        type RandomnessFromOneEpochAgo: Randomness<H256, BlockNumberFor<Self>>;

        /// Type representing the weight of this pallet.
        type WeightInfo: WeightInfo;
    }

    // STORAGE //

    /// The number of blocks before the next epoch.
    #[pallet::storage]
    pub(super) type NexEpochHookIn<T: Config> = StorageValue<_, u8, ValueQuery>;

    /// The request ID.
    #[pallet::storage]
    pub(super) type RequestIdProvider<T: Config> = StorageValue<_, RequestId, ValueQuery>;

    /// The requests that will be fulfilled at the next block.
    #[pallet::storage]
    #[pallet::getter(fn requests_ready_at_next_block)]
    pub type RequestsReadyAtNextBlock<T: Config> = StorageValue<_, Vec<Request>, ValueQuery>;

    /// The requests that will be fulfilled at the next epoch.
    #[pallet::storage]
    #[pallet::getter(fn requests_ready_at_epoch)]
    pub type RequestsReadyAtEpoch<T: Config> =
        StorageMap<_, Twox64Concat, u64, Vec<Request>, ValueQuery>;

    /// The requests being processed.
    #[pallet::storage]
    #[pallet::getter(fn requests_ids)]
    pub type RequestsIds<T: Config> =
        CountedStorageMap<_, Twox64Concat, RequestId, (), OptionQuery>;

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event {
        /// A request for randomness was fulfilled.
        FilledRandomness {
            request_id: RequestId,
            randomness: H256,
        },
        /// A request for randomness was made.
        RequestedRandomness {
            request_id: RequestId,
            salt: H256,
            r#type: RandomnessType,
        },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Request randomness queue is full.
        QueueFull,
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request randomness.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::request())]
        pub fn request(
            origin: OriginFor<T>,
            randomness_type: RandomnessType,
            salt: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let request_id = Self::do_request(&who, randomness_type, salt)?;

            Self::deposit_event(Event::RequestedRandomness {
                request_id,
                salt,
                r#type: randomness_type,
            });

            Ok(())
        }
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_: BlockNumberFor<T>) -> Weight {
            // Overhead to process an empty request
            let mut total_weight = T::WeightInfo::on_initialize(0);

            for Request { request_id, salt } in RequestsReadyAtNextBlock::<T>::take() {
                let randomness = T::ParentBlockRandomness::random(salt.as_ref())
                    .0
                    .unwrap_or_default();
                RequestsIds::<T>::remove(request_id);
                total_weight += T::OnFilledRandomness::on_filled_randomness(request_id, randomness);
                Self::deposit_event(Event::FilledRandomness {
                    request_id,
                    randomness,
                });
                // Weight to process on request
                total_weight +=
                    T::WeightInfo::on_initialize(2).saturating_sub(T::WeightInfo::on_initialize(1));
            }

            let next_epoch_hook_in = NexEpochHookIn::<T>::mutate(|next_in| {
                core::mem::replace(next_in, next_in.saturating_sub(1))
            });
            if next_epoch_hook_in == 1 {
                for Request { request_id, salt } in
                    RequestsReadyAtEpoch::<T>::take(T::GetCurrentEpochIndex::get())
                {
                    let randomness = T::RandomnessFromOneEpochAgo::random(salt.as_ref()).0;
                    RequestsIds::<T>::remove(request_id);
                    total_weight +=
                        T::OnFilledRandomness::on_filled_randomness(request_id, randomness);
                    Self::deposit_event(Event::FilledRandomness {
                        request_id,
                        randomness,
                    });
                    // Weight to process on request
                    total_weight += T::WeightInfo::on_initialize_epoch(2)
                        .saturating_sub(T::WeightInfo::on_initialize_epoch(1));
                }
            }

            total_weight
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Initiates a randomness request with specified parameters.
        pub fn do_request(
            requestor: &T::AccountId,
            randomness_type: RandomnessType,
            salt: H256,
        ) -> Result<RequestId, DispatchError> {
            // Verify phase
            ensure!(
                RequestsIds::<T>::count() < T::MaxRequests::get(),
                Error::<T>::QueueFull
            );

            Self::pay_request(requestor)?;

            // Apply phase
            Ok(Self::apply_request(randomness_type, salt))
        }

        /// Forcefully initiates a randomness request using the specified parameters.
        pub fn force_request(randomness_type: RandomnessType, salt: H256) -> RequestId {
            Self::apply_request(randomness_type, salt)
        }

        /// Set the next epoch hook value to 5.
        pub fn on_new_epoch() {
            NexEpochHookIn::<T>::put(5)
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Withdraw funds from the requestor's account to pay for a request.
        fn pay_request(requestor: &T::AccountId) -> DispatchResult {
            let imbalance = T::Currency::withdraw(
                requestor,
                T::RequestPrice::get(),
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )?;
            T::OnUnbalanced::on_unbalanced(imbalance);
            Ok(())
        }

        /// Apply a randomness request with the specified type and salt.
        fn apply_request(randomness_type: RandomnessType, salt: H256) -> RequestId {
            let request_id = RequestIdProvider::<T>::mutate(|next_request_id| {
                core::mem::replace(next_request_id, next_request_id.saturating_add(1))
            });
            RequestsIds::<T>::insert(request_id, ());
            let current_epoch = T::GetCurrentEpochIndex::get();
            match randomness_type {
                RandomnessType::RandomnessFromPreviousBlock => {
                    RequestsReadyAtNextBlock::<T>::append(Request { request_id, salt });
                }
                RandomnessType::RandomnessFromOneEpochAgo => {
                    if NexEpochHookIn::<T>::get() > 1 {
                        RequestsReadyAtEpoch::<T>::append(
                            current_epoch + 3,
                            Request { request_id, salt },
                        );
                    } else {
                        RequestsReadyAtEpoch::<T>::append(
                            current_epoch + 2,
                            Request { request_id, salt },
                        );
                    }
                }
                RandomnessType::RandomnessFromTwoEpochsAgo => {
                    if NexEpochHookIn::<T>::get() > 1 {
                        RequestsReadyAtEpoch::<T>::append(
                            current_epoch + 4,
                            Request { request_id, salt },
                        );
                    } else {
                        RequestsReadyAtEpoch::<T>::append(
                            current_epoch + 3,
                            Request { request_id, salt },
                        );
                    }
                }
            }
            request_id
        }
    }
}
