// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
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

use frame_support::traits::{tokens::ExistenceRequirement, Currency};
use sp_arithmetic::{
    per_things::Permill,
    traits::{CheckedSub, One, Saturating, Zero},
};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

const OFFCHAIN_PREFIX_UD_HISTORY: &[u8] = b"ud::history::";

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::StorageVersion;
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
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
        #[pallet::constant]
        /// Universal dividend creation period
        type UdCreationPeriod: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Universal dividend first reevaluation (in block number)
        /// Must be leess than UdReevalPeriodInBlocks
        type UdFirstReeval: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Universal dividend reevaluation period (in number of creation period)
        type UdReevalPeriod: Get<BalanceOf<Self>>;
        #[pallet::constant]
        /// Universal dividend reevaluation period in number of blocks
        /// Must be equal to UdReevalPeriod * UdCreationPeriod
        type UdReevalPeriodInBlocks: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// The number of units to divide the amounts expressed in number of UDs
        /// Example: If you wish to express the UD amounts with a maximum precision of the order
        /// of the milliUD, choose 1000
        type UnitsPerUd: Get<BalanceOf<Self>>;
    }

    // STORAGE //

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
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
    #[pallet::getter(fn current_ud)]
    pub type CurrentUdStorage<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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

            <CurrentUdStorage<T>>::put(self.first_ud);
            <MonetaryMassStorage<T>>::put(self.initial_monetary_mass);
        }
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if (n % T::UdCreationPeriod::get()).is_zero() {
                let current_members_count = T::MembersCount::get();
                if (n % T::UdReevalPeriodInBlocks::get()).checked_sub(&T::UdFirstReeval::get())
                    == Some(Zero::zero())
                {
                    Self::reeval_ud(current_members_count)
                        + Self::create_ud(current_members_count, n)
                } else {
                    Self::create_ud(current_members_count, n)
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
    pub enum Event<T: Config> {
        /// A new universal dividend is created
        /// [amout, members_count]
        NewUdCreated {
            amount: BalanceOf<T>,
            monetary_mass: BalanceOf<T>,
            members_count: BalanceOf<T>,
        },
        /// The universal dividend has been re-evaluated
        /// [new_ud_amount, monetary_mass, members_count]
        UdReevalued {
            new_ud_amount: BalanceOf<T>,
            monetary_mass: BalanceOf<T>,
            members_count: BalanceOf<T>,
        },
    }

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        fn create_ud(members_count: BalanceOf<T>, n: T::BlockNumber) -> Weight {
            let total_weight: Weight = 0;

            let ud_amount = <CurrentUdStorage<T>>::try_get().expect("corrupted storage");
            let monetary_mass = <MonetaryMassStorage<T>>::try_get().expect("corrupted storage");

            for account_id in T::MembersIds::get() {
                T::Currency::deposit_creating(&account_id, ud_amount);
                Self::write_ud_history(n, account_id, ud_amount);
            }

            let new_monetary_mass =
                monetary_mass.saturating_add(ud_amount.saturating_mul(members_count));
            MonetaryMassStorage::<T>::put(new_monetary_mass);
            Self::deposit_event(Event::NewUdCreated {
                amount: ud_amount,
                members_count,
                monetary_mass: new_monetary_mass,
            });

            total_weight
        }
        fn do_transfer_ud(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            value: BalanceOf<T>,
            existence_requirement: ExistenceRequirement,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            let ud_amount = <CurrentUdStorage<T>>::try_get()
                .map_err(|_| DispatchError::Other("corrupted storage"))?;
            T::Currency::transfer(
                &who,
                &dest,
                value.saturating_mul(ud_amount) / T::UnitsPerUd::get(),
                existence_requirement,
            )?;
            Ok(().into())
        }
        fn reeval_ud(members_count: BalanceOf<T>) -> Weight {
            let total_weight: Weight = 0;

            let ud_amount = <CurrentUdStorage<T>>::try_get().expect("corrupted storage");

            let monetary_mass = <MonetaryMassStorage<T>>::try_get().expect("corrupted storage");

            let new_ud_amount = Self::reeval_ud_formula(
                ud_amount,
                T::SquareMoneyGrowthRate::get(),
                monetary_mass,
                members_count,
                T::UdReevalPeriod::get(),
            );

            <CurrentUdStorage<T>>::put(new_ud_amount);

            Self::deposit_event(Event::UdReevalued {
                new_ud_amount,
                monetary_mass,
                members_count,
            });

            total_weight
        }
        fn reeval_ud_formula(
            ud_t: BalanceOf<T>,
            c_square: Permill,
            monetary_mass: BalanceOf<T>,
            mut members_count: BalanceOf<T>,
            count_uds_beetween_two_reevals: BalanceOf<T>, // =(dt/udFrequency)
        ) -> BalanceOf<T> {
            // Ensure that we not divide by zero
            if members_count.is_zero() {
                members_count = One::one();
            }

            // UD(t+1) = UD(t) + c² (M(t+1) / N(t+1)) / (dt/udFrequency)
            ud_t + (c_square * monetary_mass) / (members_count * count_uds_beetween_two_reevals)
        }
        fn write_ud_history(n: T::BlockNumber, account_id: T::AccountId, ud_amount: BalanceOf<T>) {
            let mut key = Vec::with_capacity(57);
            key.extend_from_slice(OFFCHAIN_PREFIX_UD_HISTORY);
            account_id.encode_to(&mut key);
            n.encode_to(&mut key);
            sp_io::offchain_index::set(key.as_ref(), ud_amount.encode().as_ref());
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Transfer some liquid free balance to another account, in milliUD.
        #[pallet::weight(0)]
        pub fn transfer_ud(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::do_transfer_ud(origin, dest, value, ExistenceRequirement::AllowDeath)
        }

        /// Transfer some liquid free balance to another account, in milliUD.
        #[pallet::weight(0)]
        pub fn transfer_ud_keep_alive(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::do_transfer_ud(origin, dest, value, ExistenceRequirement::KeepAlive)
        }
    }
}
