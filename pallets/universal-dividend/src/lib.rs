// Copyright 2021 Axiom-Team
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

mod benchmarking;
mod compute_claim_uds;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;
mod weights;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::traits::{tokens::ExistenceRequirement, Currency, OnTimestampSet};
use sp_arithmetic::{
    per_things::Perbill,
    traits::{One, Saturating, Zero},
};
use sp_runtime::traits::{Get, MaybeSerializeDeserialize, StaticLookup};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::{StorageVersion, StoredMap};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Convert;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    //#[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        // Moment into Balance converter
        type MomentIntoBalance: Convert<Self::Moment, BalanceOf<Self>>;
        // The currency
        type Currency: Currency<Self::AccountId>;
        #[pallet::constant]
        /// Maximum number of past UD revaluations to keep in storage.
        type MaxPastReeval: Get<u32>;
        /// Somethings that must provide the number of accounts allowed to create the universal dividend
        type MembersCount: Get<BalanceOf<Self>>;
        /// Somethings that must provide the list of accounts ids allowed to create the universal dividend
        type MembersStorage: frame_support::traits::StoredMap<Self::AccountId, FirstEligibleUd>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        /// Square of the money growth rate per ud reevaluation period
        type SquareMoneyGrowthRate: Get<Perbill>;
        #[pallet::constant]
        /// Universal dividend creation period (ms)
        type UdCreationPeriod: Get<Self::Moment>;
        #[pallet::constant]
        /// Universal dividend reevaluation period (ms)
        type UdReevalPeriod: Get<Self::Moment>;
        #[pallet::constant]
        /// The number of units to divide the amounts expressed in number of UDs
        /// Example: If you wish to express the UD amounts with a maximum precision of the order
        /// of the milliUD, choose 1000
        type UnitsPerUd: Get<BalanceOf<Self>>;
        /// Pallet weights info
        type WeightInfo: WeightInfo;
        #[cfg(feature = "runtime-benchmarks")]
        type AccountIdOf: Convert<u32, Option<Self::AccountId>>;
    }

    // STORAGE //

    /// Current UD amount
    #[pallet::storage]
    #[pallet::getter(fn current_ud)]
    pub type CurrentUd<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultForCurrentUdIndex() -> UdIndex {
        1
    }

    /// Current UD index
    #[pallet::storage]
    #[pallet::getter(fn ud_index)]
    pub type CurrentUdIndex<T: Config> =
        StorageValue<_, UdIndex, ValueQuery, DefaultForCurrentUdIndex>;

    #[cfg(test)]
    #[pallet::storage]
    pub type TestMembers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        FirstEligibleUd,
        ValueQuery,
        GetDefault,
        ConstU32<300_000>,
    >;

    /// Total quantity of money created by universal dividend (does not take into account the possible destruction of money)
    #[pallet::storage]
    #[pallet::getter(fn total_money_created)]
    pub type MonetaryMass<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Next UD reevaluation
    #[pallet::storage]
    #[pallet::getter(fn next_reeval)]
    pub type NextReeval<T: Config> = StorageValue<_, T::Moment, OptionQuery>;

    /// Next UD creation
    #[pallet::storage]
    #[pallet::getter(fn next_ud)]
    pub type NextUd<T: Config> = StorageValue<_, T::Moment, OptionQuery>;

    /// Past UD reevaluations
    #[pallet::storage]
    #[pallet::getter(fn past_reevals)]
    pub type PastReevals<T: Config> =
        StorageValue<_, BoundedVec<(UdIndex, BalanceOf<T>), T::MaxPastReeval>, ValueQuery>;

    // GENESIS

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config>
    where
        <T as pallet_timestamp::Config>::Moment: MaybeSerializeDeserialize,
    {
        /// moment of the first UD reeval
        // If None, it will be set to one period after the first block with a timestamp
        pub first_reeval: Option<T::Moment>,
        /// moment of the first UD generation
        // If None, it will be set to one period after the first block with a timestamp
        pub first_ud: Option<T::Moment>,
        /// initial monetary mass (should match total issuance)
        pub initial_monetary_mass: BalanceOf<T>,
        /// accounts of initial members
        // (only for test purpose)
        #[cfg(test)]
        pub initial_members: Vec<T::AccountId>,
        /// value of the first UD
        /// expressed in amount of currency
        pub ud: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T>
    where
        <T as pallet_timestamp::Config>::Moment: MaybeSerializeDeserialize,
    {
        fn default() -> Self {
            Self {
                first_reeval: None,
                first_ud: None,
                initial_monetary_mass: Default::default(),
                #[cfg(test)]
                initial_members: Default::default(),
                ud: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
    where
        <T as pallet_timestamp::Config>::Moment: MaybeSerializeDeserialize,
    {
        fn build(&self) {
            assert!(!self.ud.is_zero());

            <CurrentUd<T>>::put(self.ud);
            // totalissuance should be updated to the same amount
            <MonetaryMass<T>>::put(self.initial_monetary_mass);

            NextReeval::<T>::set(self.first_reeval);
            NextUd::<T>::set(self.first_ud);
            let mut past_reevals = BoundedVec::default();
            past_reevals
                .try_push((1, self.ud))
                .expect("MaxPastReeval should be greather than zero");
            PastReevals::<T>::put(past_reevals);

            #[cfg(test)]
            {
                for member in &self.initial_members {
                    TestMembers::<T>::insert(member, FirstEligibleUd::min());
                }
            }
        }
    }

    // EVENTS //

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new universal dividend is created.
        NewUdCreated {
            amount: BalanceOf<T>,
            index: UdIndex,
            monetary_mass: BalanceOf<T>,
            members_count: BalanceOf<T>,
        },
        /// The universal dividend has been re-evaluated.
        UdReevalued {
            new_ud_amount: BalanceOf<T>,
            monetary_mass: BalanceOf<T>,
            members_count: BalanceOf<T>,
        },
        /// DUs were automatically transferred as part of a member removal.
        UdsAutoPaid {
            count: UdIndex,
            total: BalanceOf<T>,
            who: T::AccountId,
        },
        /// A member claimed his UDs.
        UdsClaimed {
            count: UdIndex,
            total: BalanceOf<T>,
            who: T::AccountId,
        },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// This account is not allowed to claim UDs.
        AccountNotAllowedToClaimUds,
    }

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        /// create universal dividend
        pub(crate) fn create_ud(members_count: BalanceOf<T>) {
            // get current value of UD and monetary mass
            let ud_amount = <CurrentUd<T>>::get();
            let monetary_mass = <MonetaryMass<T>>::get();

            // Increment ud index
            let ud_index = CurrentUdIndex::<T>::mutate(|next_ud_index| {
                core::mem::replace(next_ud_index, next_ud_index.saturating_add(1))
            });

            // compute the new monetary mass
            let new_monetary_mass =
                monetary_mass.saturating_add(ud_amount.saturating_mul(members_count));

            // update the storage value of the monetary mass
            MonetaryMass::<T>::put(new_monetary_mass);

            // emit an event to inform blockchain users that the holy UNIVERSAL DIVIDEND was created
            Self::deposit_event(Event::NewUdCreated {
                amount: ud_amount,
                index: ud_index,
                members_count,
                monetary_mass: new_monetary_mass,
            });
        }

        /// claim all due universal dividend at a time
        fn do_claim_uds(who: &T::AccountId) -> DispatchResultWithPostInfo {
            T::MembersStorage::try_mutate_exists(who, |maybe_first_eligible_ud| {
                if let Some(FirstEligibleUd(Some(ref mut first_ud_index))) = maybe_first_eligible_ud
                {
                    let current_ud_index = CurrentUdIndex::<T>::get();
                    if first_ud_index.get() >= current_ud_index {
                        DispatchResultWithPostInfo::Ok(().into())
                    } else {
                        let (uds_count, uds_total) = compute_claim_uds::compute_claim_uds(
                            current_ud_index,
                            first_ud_index.get(),
                            PastReevals::<T>::get().into_iter(),
                        );
                        let _ = core::mem::replace(
                            first_ud_index,
                            core::num::NonZeroU16::new(current_ud_index)
                                .expect("unrechable because current_ud_index is never zero."),
                        );
                        T::Currency::deposit_creating(who, uds_total);
                        Self::deposit_event(Event::UdsClaimed {
                            count: uds_count,
                            total: uds_total,
                            who: who.clone(),
                        });
                        Ok(().into())
                    }
                } else {
                    Err(Error::<T>::AccountNotAllowedToClaimUds.into())
                }
            })
        }

        /// like balance.transfer, but give an amount in UD
        fn do_transfer_ud(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            value: BalanceOf<T>,
            existence_requirement: ExistenceRequirement,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            let ud_amount = <CurrentUd<T>>::get();
            T::Currency::transfer(
                &who,
                &dest,
                value.saturating_mul(ud_amount) / T::UnitsPerUd::get(),
                existence_requirement,
            )?;
            Ok(().into())
        }

        /// reevaluate the value of the universal dividend
        pub(crate) fn reeval_ud(members_count: BalanceOf<T>) {
            // get current value and monetary mass
            let ud_amount = <CurrentUd<T>>::get();
            let monetary_mass = <MonetaryMass<T>>::get();

            // compute new value
            let new_ud_amount = Self::reeval_ud_formula(
                ud_amount,
                T::SquareMoneyGrowthRate::get(),
                monetary_mass,
                members_count,
                T::MomentIntoBalance::convert(
                    T::UdReevalPeriod::get() / T::UdCreationPeriod::get(),
                ),
            );

            // update the storage value and the history of past reevals
            CurrentUd::<T>::put(new_ud_amount);
            PastReevals::<T>::mutate(|past_reevals| {
                if past_reevals.len() == T::MaxPastReeval::get() as usize {
                    past_reevals.remove(0);
                }
                past_reevals
                    .try_push((CurrentUdIndex::<T>::get(), new_ud_amount))
                    .expect("Unreachable, because we removed an element just before.")
            });

            Self::deposit_event(Event::UdReevalued {
                new_ud_amount,
                monetary_mass,
                members_count,
            });
        }

        /// formula for Universal Dividend reevaluation
        fn reeval_ud_formula(
            ud_t: BalanceOf<T>,
            c_square: Perbill,
            monetary_mass: BalanceOf<T>,
            mut members_count: BalanceOf<T>,
            count_uds_beetween_two_reevals: BalanceOf<T>, // =(dt/udFrequency)
        ) -> BalanceOf<T> {
            // Ensure that we do not divide by zero
            if members_count.is_zero() {
                members_count = One::one();
            }

            // UD(t+1) = UD(t) + c² (M(t+1) / N(t+1)) / (dt/udFrequency)
            ud_t + (c_square * monetary_mass) / (members_count * count_uds_beetween_two_reevals)
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Claim Universal Dividends
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::claim_uds(T::MaxPastReeval::get()))]
        pub fn claim_uds(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::do_claim_uds(&who)
        }
        /// Transfer some liquid free balance to another account, in milliUD.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_ud())]
        pub fn transfer_ud(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::do_transfer_ud(origin, dest, value, ExistenceRequirement::AllowDeath)
        }

        /// Transfer some liquid free balance to another account, in milliUD.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_ud_keep_alive())]
        pub fn transfer_ud_keep_alive(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::do_transfer_ud(origin, dest, value, ExistenceRequirement::KeepAlive)
        }
    }

    // PUBLIC FUNCTIONS

    impl<T: Config> Pallet<T> {
        pub fn init_first_eligible_ud() -> FirstEligibleUd {
            CurrentUdIndex::<T>::get().into()
        }
        /// function to call when removing a member
        /// auto-claims UDs
        pub fn on_removed_member(first_ud_index: UdIndex, who: &T::AccountId) -> Weight {
            let current_ud_index = CurrentUdIndex::<T>::get();
            if first_ud_index < current_ud_index {
                let (uds_count, uds_total) = compute_claim_uds::compute_claim_uds(
                    current_ud_index,
                    first_ud_index,
                    PastReevals::<T>::get().into_iter(),
                );
                T::Currency::deposit_creating(who, uds_total);
                Self::deposit_event(Event::UdsAutoPaid {
                    count: uds_count,
                    total: uds_total,
                    who: who.clone(),
                });
                <T as pallet::Config>::WeightInfo::on_removed_member(first_ud_index as u32)
            } else {
                <T as pallet::Config>::WeightInfo::on_removed_member(0)
            }
        }
    }
}

impl<T: Config> OnTimestampSet<T::Moment> for Pallet<T>
where
    <T as pallet_timestamp::Config>::Moment: MaybeSerializeDeserialize,
{
    fn on_timestamp_set(moment: T::Moment) {
        let next_ud = NextUd::<T>::get().unwrap_or_else(|| {
            let next_ud = moment.saturating_add(T::UdCreationPeriod::get());
            NextUd::<T>::put(next_ud);
            next_ud
        });
        if moment >= next_ud {
            let current_members_count = T::MembersCount::get();
            let next_reeval = NextReeval::<T>::get().unwrap_or_else(|| {
                let next_reeval = moment.saturating_add(T::UdReevalPeriod::get());
                NextReeval::<T>::put(next_reeval);
                next_reeval
            });
            // Reevaluation may happen later than expected, but this has no effect before a new UD
            // is created. This is why we can check for reevaluation only when creating UD.
            if moment >= next_reeval {
                NextReeval::<T>::put(next_reeval.saturating_add(T::UdReevalPeriod::get()));
                Self::reeval_ud(current_members_count);
            }
            Self::create_ud(current_members_count);
            NextUd::<T>::put(next_ud.saturating_add(T::UdCreationPeriod::get()));
        }
    }
}
