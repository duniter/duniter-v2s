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

//! # Duniter Universal Dividend Pallet
//!
//! One of Duniter's core features is the Universal Dividend (UD), which operates based on the Relative Theory of Money. The UD serves both as a daily monetary creation mechanism and a unit of measure within the Duniter ecosystem.
//!
//! ## Overview
//!
//! This pallet enables:
//! - Creation of Universal Dividends (UD) as a daily monetary issuance and measure unit.
//! - Transfer of currency denominated in UD between accounts.
//!
//! **Note**: The UD is not automatically created daily for every account due to resource constraints. Instead, members must claim their UD using a specific extrinsic.

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod compute_claim_uds;
mod runtime_api;
mod types;
mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
use duniter_primitives::Idty;

pub use pallet::*;
pub use runtime_api::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::traits::{
    fungible::{self, Balanced, Inspect, Mutate},
    tokens::{Fortitude, Precision, Preservation},
    OnTimestampSet, ReservableCurrency,
};
use sp_arithmetic::{
    per_things::Perbill,
    traits::{One, Saturating, Zero},
};
use sp_runtime::traits::{Get, MaybeSerializeDeserialize, StaticLookup};

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{StorageVersion, StoredMap},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Convert;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type BalanceOf<T> = <<T as Config>::Currency as fungible::Inspect<AccountIdOf<T>>>::Balance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// Something that convert a Moment inot a Balance.
        type MomentIntoBalance: Convert<Self::Moment, BalanceOf<Self>>;

        /// The currency type used in this pallet.
        type Currency: fungible::Balanced<Self::AccountId>
            + fungible::Mutate<Self::AccountId>
            + fungible::Inspect<Self::AccountId>
            + ReservableCurrency<Self::AccountId>;

        /// Maximum number of past UD revaluations to keep in storage.
        #[pallet::constant]
        type MaxPastReeval: Get<u32>;

        /// Provides the number of accounts allowed to create the universal dividend.
        type MembersCount: Get<BalanceOf<Self>>;

        /// Storage for mapping AccountId to their first eligible UD creation time.
        type MembersStorage: frame_support::traits::StoredMap<Self::AccountId, FirstEligibleUd>;

        /// The overarching event type for this pallet.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Square of the money growth rate per UD reevaluation period.
        #[pallet::constant]
        type SquareMoneyGrowthRate: Get<Perbill>;

        /// Universal dividend creation period in milliseconds.
        #[pallet::constant]
        type UdCreationPeriod: Get<Self::Moment>;

        /// Universal dividend reevaluation period in milliseconds.
        #[pallet::constant]
        type UdReevalPeriod: Get<Self::Moment>;

        /// Type representing the weight of this pallet.
        type WeightInfo: WeightInfo;

        /// Something that gives the IdtyIndex of an AccountId and reverse, used for benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type IdtyAttr: duniter_primitives::Idty<u32, Self::AccountId>;
    }

    // STORAGE //

    /// The current Universal Dividend value.
    #[pallet::storage]
    #[pallet::getter(fn current_ud)]
    pub type CurrentUd<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The default index for the current Universal Dividend.
    #[pallet::type_value]
    pub fn DefaultForCurrentUdIndex() -> UdIndex {
        1
    }

    /// The current Universal Dividend index.
    #[pallet::storage]
    #[pallet::getter(fn ud_index)]
    pub type CurrentUdIndex<T: Config> =
        StorageValue<_, UdIndex, ValueQuery, DefaultForCurrentUdIndex>;

    #[cfg(test)]
    #[pallet::storage]
    // UD should be linked to idtyid instead of accountid
    // if it is convenient in test, why not have it in runtime also?
    // storing it in idty_value.data is strange
    pub type TestMembers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        FirstEligibleUd,
        ValueQuery,
        GetDefault,
        ConstU32<300_000>,
    >;

    /// The total quantity of money created by Universal Dividend, excluding potential money destruction.
    #[pallet::storage]
    #[pallet::getter(fn total_money_created)]
    pub type MonetaryMass<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The next Universal Dividend re-evaluation.
    #[pallet::storage]
    #[pallet::getter(fn next_reeval)]
    pub type NextReeval<T: Config> = StorageValue<_, T::Moment, OptionQuery>;

    /// The next Universal Dividend creation.
    #[pallet::storage]
    #[pallet::getter(fn next_ud)]
    pub type NextUd<T: Config> = StorageValue<_, T::Moment, OptionQuery>;

    /// The past Universal Dividend re-evaluations.
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
                ud: BalanceOf::<T>::one(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T>
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
                                .expect("unreachable because current_ud_index is never zero."),
                        );
                        // Currency is issued here
                        let actual_total = T::Currency::mint_into(who, uds_total)?;
                        Self::deposit_event(Event::UdsClaimed {
                            count: uds_count,
                            total: actual_total,
                            who: who.clone(),
                        });
                        Ok(().into())
                    }
                } else {
                    Err(Error::<T>::AccountNotAllowedToClaimUds.into())
                }
            })
        }

        /// like balance.transfer, but give an amount in milliUD
        fn do_transfer_ud(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            value: BalanceOf<T>,
            preservation: Preservation,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            let ud_amount = <CurrentUd<T>>::get();
            T::Currency::transfer(
                &who,
                &dest,
                value.saturating_mul(ud_amount) / 1_000u32.into(),
                preservation,
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
        /// Claim Universal Dividends.
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
            Self::do_transfer_ud(origin, dest, value, Preservation::Expendable)
        }

        /// Transfer some liquid free balance to another account in milliUD and keep the account alive.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_ud_keep_alive())]
        pub fn transfer_ud_keep_alive(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::do_transfer_ud(origin, dest, value, Preservation::Preserve)
        }
    }

    // PUBLIC FUNCTIONS

    impl<T: Config> Pallet<T> {
        /// Initialize the first eligible Universal Dividend index.
        pub fn init_first_eligible_ud() -> FirstEligibleUd {
            CurrentUdIndex::<T>::get().into()
        }

        /// Handle the removal of a member, which automatically claims Universal Dividends.
        pub fn on_removed_member(first_ud_index: UdIndex, who: &T::AccountId) -> Weight {
            let current_ud_index = CurrentUdIndex::<T>::get();
            if first_ud_index < current_ud_index {
                let (uds_count, uds_total) = compute_claim_uds::compute_claim_uds(
                    current_ud_index,
                    first_ud_index,
                    PastReevals::<T>::get().into_iter(),
                );
                let _ = T::Currency::deposit(who, uds_total, Precision::Exact);
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

        /// Get the total balance information for an account
        ///
        /// Returns an object with three fields:
        /// - `transferable`: sum of free + unclaim_uds
        /// - `reserved`: reserved balance
        /// - `unclaim_uds`: amount of unclaimed UDs computed by compute_claim_uds
        pub fn account_balances(who: &T::AccountId) -> crate::AccountBalances<BalanceOf<T>> {
            let total_balance = T::Currency::total_balance(who);
            let reducible_balance =
                T::Currency::reducible_balance(who, Preservation::Preserve, Fortitude::Polite);

            // Calculate unclaimed UDs
            let current_ud_index = CurrentUdIndex::<T>::get();
            let maybe_first_eligible_ud = T::MembersStorage::get(who);

            let unclaim_uds =
                if let FirstEligibleUd(Some(ref first_ud_index)) = maybe_first_eligible_ud {
                    let past_reevals = PastReevals::<T>::get();
                    compute_claim_uds::compute_claim_uds(
                        current_ud_index,
                        first_ud_index.get(),
                        past_reevals.into_iter(),
                    )
                    .1
                } else {
                    Zero::zero()
                };

            crate::AccountBalances {
                total: total_balance.saturating_add(unclaim_uds),
                transferable: reducible_balance.saturating_add(unclaim_uds),
                unclaim_uds,
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
