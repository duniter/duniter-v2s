// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::traits::Currency;
use sp_arithmetic::{per_things::Permill, traits::Zero};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Universal dividend creation period
        const UD_CREATION_PERIOD: Self::BlockNumber;
        /// Universal dividend reevaluation period (in number of creation period)
        const UD_REEVAL_PERIOD: BalanceOf<Self>;
        /// Universal dividend reevaluation period in number of blocks
        /// Must be equal to UD_CREATION_PERIOD * UD_REEVAl_PERIOD
        const UD_REEVAL_PERIOD_IN_BLOCKS: Self::BlockNumber;

        // The currency
        type Currency: Currency<Self::AccountId>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Somethings that must provide the number of accounts allowed to create the universal dividend
        type MembersCount: Get<BalanceOf<Self>>;
        /// Somethings that must provide the list of accounts ids allowed to create the universal dividend
        type MembersIds: Get<Vec<<Self as frame_system::Config>::AccountId>>;
        #[pallet::constant]
        /// Square of the money growth rate per ud reevaluation period
        type SquareMoneyGrowthRate: Get<Permill>;
    }

    // STORAGE //

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // A value placed in storage that represents the current version of the Balances storage.
    // This value is used by the `on_runtime_upgrade` logic to determine whether we run
    // storage migration logic. This should match directly with the semantic versions of the Rust crate.
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub enum Releases {
        V1_0_0,
    }
    impl Default for Releases {
        fn default() -> Self {
            Releases::V1_0_0
        }
    }

    /// Storage version of the pallet.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub struct LastReeval<T: Config> {
        members_count: BalanceOf<T>,
        monetary_mass: BalanceOf<T>,
        ud_amount: BalanceOf<T>,
    }
    impl<T: Config> Default for LastReeval<T> {
        fn default() -> Self {
            Self {
                monetary_mass: Default::default(),
                members_count: Default::default(),
                ud_amount: Default::default(),
            }
        }
    }

    /// Last reevaluation
    #[pallet::storage]
    #[pallet::getter(fn last_reeval)]
    pub type LastReevalStorage<T: Config> = StorageValue<_, LastReeval<T>, ValueQuery>;

    /// Total quantity of money created by universal dividend (does not take into account the possible destruction of money)
    #[pallet::storage]
    #[pallet::getter(fn total_money_created)]
    pub type MonetaryMassStorage<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // GENESIS

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub first_ud: BalanceOf<T>,
        pub initial_monetary_mass: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                first_ud: Default::default(),
                initial_monetary_mass: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            assert!(!self.first_ud.is_zero());
            assert!(self.initial_monetary_mass >= T::Currency::total_issuance());

            <StorageVersion<T>>::put(Releases::V1_0_0);
            <LastReevalStorage<T>>::put(LastReeval {
                monetary_mass: T::Currency::total_issuance(),
                members_count: T::MembersCount::get(),
                ud_amount: self.first_ud,
            });
            <MonetaryMassStorage<T>>::put(self.initial_monetary_mass);
        }
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if (n % T::UD_CREATION_PERIOD).is_zero() {
                let current_members_count = T::MembersCount::get();
                if (n % T::UD_REEVAL_PERIOD_IN_BLOCKS).is_zero() {
                    Self::reeval_ud(current_members_count) + Self::create_ud(current_members_count)
                } else {
                    Self::create_ud(current_members_count)
                }
            } else {
                0
            }
        }
    }

    // EVENTS //

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// A new universal dividend is created
        /// [ud_amout, members_count]
        NewUdCreated(BalanceOf<T>, BalanceOf<T>),
        /// The universal dividend has been re-evaluated
        /// [new_ud_amount, monetary_mass, members_count]
        UdReevalued(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
    }

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        fn create_ud(current_members_count: BalanceOf<T>) -> Weight {
            let total_weight: Weight = 0;

            let LastReeval { ud_amount, .. } =
                <LastReevalStorage<T>>::try_get().expect("corrupted storage");
            let mut monetary_mass = <MonetaryMassStorage<T>>::try_get().expect("corrupted storage");

            for account_id in T::MembersIds::get() {
                T::Currency::deposit_creating(&account_id, ud_amount);
                monetary_mass += ud_amount;
            }

            <MonetaryMassStorage<T>>::put(monetary_mass);
            Self::deposit_event(Event::NewUdCreated(ud_amount, current_members_count));

            total_weight
        }
        fn reeval_ud(current_members_count: BalanceOf<T>) -> Weight {
            let total_weight: Weight = 0;

            let LastReeval {
                members_count,
                mut monetary_mass,
                ud_amount,
            } = <LastReevalStorage<T>>::try_get().expect("corrupted storage");

            if monetary_mass.is_zero() {
                monetary_mass = ud_amount * members_count;
            }

            let new_ud_amount = Self::reeval_ud_formula(
                ud_amount,
                T::SquareMoneyGrowthRate::get(),
                monetary_mass,
                members_count,
                T::UD_REEVAL_PERIOD,
            );

            Self::deposit_event(Event::UdReevalued(
                new_ud_amount,
                monetary_mass,
                members_count,
            ));

            let monetary_mass = <MonetaryMassStorage<T>>::try_get().expect("corrupted storage");
            <LastReevalStorage<T>>::put(LastReeval {
                members_count: current_members_count,
                monetary_mass,
                ud_amount: new_ud_amount,
            });

            total_weight
        }
        fn reeval_ud_formula(
            ud_t: BalanceOf<T>,
            c_square: Permill,
            monetary_mass: BalanceOf<T>,
            members_count: BalanceOf<T>,
            count_uds_beetween_two_reevals: BalanceOf<T>, // =(dt/udFrequency)
        ) -> BalanceOf<T> {
            // UD(t+1) = UD(t) + c² (M(t) / N(t)) / (dt/udFrequency)
            ud_t + c_square * monetary_mass / (members_count * count_uds_beetween_two_reevals)
        }
    }
}
