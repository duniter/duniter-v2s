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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::boxed_local)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub mod weights;

use frame_support::pallet_prelude::Weight;
use sp_core::H256;
use sp_std::prelude::*;

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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::{
        Currency, ExistenceRequirement, OnUnbalanced, Randomness, StorageVersion, WithdrawReasons,
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H256;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config<Hash = H256> {
        // The currency
        type Currency: Currency<Self::AccountId>;
        /// Get the current epoch index
        type GetCurrentEpochIndex: Get<u64>;
        /// Maximum number of not yet filled requests
        #[pallet::constant]
        type MaxRequests: Get<u32>;
        /// The price of a request
        #[pallet::constant]
        type RequestPrice: Get<BalanceOf<Self>>;
        /// On filled randomness
        type OnFilledRandomness: OnFilledRandomness;
        /// Handler for the unbalanced reduction when the requestor pays fees.
        type OnUnbalanced: OnUnbalanced<NegativeImbalanceOf<Self>>;
        /// A safe source of randomness from the parent block
        type ParentBlockRandomness: Randomness<Option<H256>, Self::BlockNumber>;
        /// A safe source of randomness from one epoch ago
        type RandomnessFromOneEpochAgo: Randomness<H256, Self::BlockNumber>;
        /// The overarching event type.
        type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
    }

    // STORAGE //

    #[pallet::storage]
    pub(super) type NexEpochHookIn<T: Config> = StorageValue<_, u8, ValueQuery>;

    #[pallet::storage]
    pub(super) type RequestIdProvider<T: Config> = StorageValue<_, RequestId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn requests_ready_at_next_block)]
    pub type RequestsReadyAtNextBlock<T: Config> = StorageValue<_, Vec<Request>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn requests_ready_at_epoch)]
    pub type RequestsReadyAtEpoch<T: Config> =
        StorageMap<_, Twox64Concat, u64, Vec<Request>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn requests_ids)]
    pub type RequestsIds<T: Config> =
        CountedStorageMap<_, Twox64Concat, RequestId, (), OptionQuery>;

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event {
        /// Filled randomness
        FilledRandomness {
            request_id: RequestId,
            randomness: H256,
        },
        /// Requested randomness
        RequestedRandomness {
            request_id: RequestId,
            salt: H256,
            r#type: RandomnessType,
        },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// The queue is full, pleasy retry later
        FullQueue,
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request a randomness
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
        fn on_initialize(_: T::BlockNumber) -> Weight {
            let request_weight = T::WeightInfo::on_initialize(T::MaxRequests::get());

            let mut total_weight = Weight::zero();

            total_weight += request_weight;
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
                total_weight += request_weight;
            }

            let next_epoch_hook_in = NexEpochHookIn::<T>::mutate(|next_in| {
                core::mem::replace(next_in, next_in.saturating_sub(1))
            });
            if next_epoch_hook_in == 1 {
                total_weight += request_weight;
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
                    total_weight += request_weight;
                }
            }

            total_weight
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn do_request(
            requestor: &T::AccountId,
            randomness_type: RandomnessType,
            salt: H256,
        ) -> Result<RequestId, DispatchError> {
            // Verify phase
            ensure!(
                RequestsIds::<T>::count() < T::MaxRequests::get(),
                Error::<T>::FullQueue
            );

            Self::pay_request(requestor)?;

            // Apply phase
            Ok(Self::apply_request(randomness_type, salt))
        }
        pub fn force_request(randomness_type: RandomnessType, salt: H256) -> RequestId {
            Self::apply_request(randomness_type, salt)
        }
        pub fn on_new_epoch() {
            NexEpochHookIn::<T>::put(5)
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        fn pay_request(requestor: &T::AccountId) -> DispatchResult {
            let imbalance = T::Currency::withdraw(
                requestor,
                T::RequestPrice::get(),
                WithdrawReasons::FEE,
                ExistenceRequirement::KeepAlive,
            )?;
            T::OnUnbalanced::on_unbalanced(imbalance);
            Ok(())
        }
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
