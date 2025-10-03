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

//! # Duniter Quota Pallet
//!
//! ## Overview
//!
//! This pallet is designed to manage transaction fee refunds based on quotas allocated to identities within the Duniter identity system. Quotas are linked to transaction fees, ensuring efficient handling of fee refunds when transactions occur.
//!
//! ## Refund Mechanism
//!
//! When a transaction is processed:
//! - The `OnChargeTransaction` implementation in the `frame-executive` pallet is called.
//! - The `OnChargeTransaction` implementation in the `duniter-account` pallet checks if the paying account is linked to an identity.
//! - If linked, the `request_refund` function in the `quota` pallet evaluates the eligibility for fee refund based on the identity's quota.
//! - Eligible refunds are added to the `RefundQueue`, managed by `process_refund_queue` during the `on_idle` phase.
//! - Refunds are processed with `try_refund`, using quotas to refund fees via `spend_quota`, and then executing the refund through `do_refund` by transferring currency from the `RefundAccount` back to the paying account.
//!
//! ## Conditions for Refund
//!
//! Refunds are executed under the following conditions:
//! 1. The paying account is linked to an identity.
//! 2. Quotas are allocated to the identity and have a non-zero value after updates.

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod traits;
mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use scale_info::prelude::vec::Vec;
use sp_runtime::traits::Zero;

pub use pallet::*;
pub use traits::*;
pub use weights::WeightInfo;

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub const MAX_QUEUED_REFUNDS: u32 = 256;

    // Currency used for quota is the one of pallet balances
    pub type CurrencyOf<T> = pallet_balances::Pallet<T>;
    // Balance used for quota is the one associated to balance currency
    pub type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    // identity id is pallet identity idty_index
    pub type IdtyId<T> = <T as pallet_identity::Config>::IdtyIndex;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // CONFIG //
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_balances::Config + pallet_identity::Config
    {
        /// Number of blocks after which the maximum quota is replenished.
        type ReloadRate: Get<BlockNumberFor<Self>>;

        /// Maximum amount of quota an identity can receive.
        type MaxQuota: Get<BalanceOf<Self>>;

        /// Account used to refund fees.
        #[pallet::constant]
        type RefundAccount: Get<Self::AccountId>;

        /// Type representing the weight of this pallet.
        type WeightInfo: WeightInfo;
    }

    // TYPES //
    /// Represents a refund.
    #[derive(Encode, Decode, Clone, TypeInfo, Debug, PartialEq, MaxEncodedLen)]
    pub struct Refund<AccountId, IdtyId, Balance> {
        /// Account to refund.
        pub account: AccountId,
        /// Identity to use quota.
        pub identity: IdtyId,
        /// Amount of refund.
        pub amount: Balance,
    }

    /// Represents a quota.
    #[derive(Encode, Decode, Clone, TypeInfo, Debug, PartialEq, MaxEncodedLen)]
    pub struct Quota<BlockNumber, Balance> {
        /// Block number of the last quota used.
        pub last_use: BlockNumber,
        /// Amount of remaining quota.
        pub amount: Balance,
    }

    // STORAGE //
    /// The quota for each identity.
    #[pallet::storage]
    #[pallet::getter(fn quota)]
    pub type IdtyQuota<T: Config> =
        StorageMap<_, Twox64Concat, IdtyId<T>, Quota<BlockNumberFor<T>, BalanceOf<T>>, OptionQuery>;

    /// The fees waiting to be refunded.
    #[pallet::storage]
    pub type RefundQueue<T: Config> = StorageValue<
        _,
        BoundedVec<Refund<T::AccountId, IdtyId<T>, BalanceOf<T>>, ConstU32<MAX_QUEUED_REFUNDS>>,
        ValueQuery,
    >;

    // EVENTS //
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Transaction fees were refunded.
        Refunded {
            who: T::AccountId,
            identity: IdtyId<T>,
            amount: BalanceOf<T>,
        },
        /// No more quota available for refund.
        NoQuotaForIdty(IdtyId<T>),
        /// No more currency available for refund.
        /// This scenario should never occur if the fees are intended for the refund account.
        NoMoreCurrencyForRefund,
        /// The refund has failed.
        /// This scenario should rarely occur, except when the account was destroyed in the interim between the request and the refund.
        RefundFailed(T::AccountId),
        /// Refund queue was full.
        RefundQueueFull,
    }

    // This pallet only contains the `on_idle` hook and no call.
    // Hooks are infallible by definition, so there are no error. To monitor no-ops
    // from inside the quota pallet, we use events as mentioned in
    // https://substrate.stackexchange.com/questions/9854/emitting-errors-from-hooks-like-on-initialize

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Estimates the quota refund amount for an identity
        /// The estimation simulate a refund request at the current block
        pub fn estimate_quota_refund(idty_index: IdtyId<T>) -> BalanceOf<T> {
            if is_eligible_for_refund::<T>(idty_index) {
                if let Some(quota) = IdtyQuota::<T>::get(idty_index).as_mut() {
                    Self::update_quota(quota);
                    quota.amount
                } else {
                    Zero::zero()
                }
            } else {
                Zero::zero()
            }
        }
    }

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        /// Adds a new refund request to the refund queue.
        pub fn queue_refund(refund: Refund<T::AccountId, IdtyId<T>, BalanceOf<T>>) {
            if RefundQueue::<T>::mutate(|v| v.try_push(refund)).is_err() {
                Self::deposit_event(Event::RefundQueueFull);
            }
        }

        /// Attempts to process a refund using available quota.
        pub fn try_refund(queued_refund: Refund<T::AccountId, IdtyId<T>, BalanceOf<T>>) -> Weight {
            // get the amount of quota that identity is able to spend
            let amount = Self::spend_quota(queued_refund.identity, queued_refund.amount);
            if amount.is_zero() {
                // partial weight
                return <T as pallet::Config>::WeightInfo::spend_quota();
            }
            // only perform refund if amount is not null
            Self::do_refund(queued_refund, amount);
            // total weight
            <T as pallet::Config>::WeightInfo::spend_quota()
                .saturating_add(<T as pallet::Config>::WeightInfo::do_refund())
        }

        /// Performs a refund operation for a specified non-null amount from the refund account to the requester's account.
        // opti: more accurate estimation of consumed weight
        pub fn do_refund(
            queued_refund: Refund<T::AccountId, IdtyId<T>, BalanceOf<T>>,
            amount: BalanceOf<T>,
        ) {
            // take money from refund account
            let res = CurrencyOf::<T>::withdraw(
                &T::RefundAccount::get(),
                amount,
                frame_support::traits::WithdrawReasons::FEE, // a fee but in reverse
                ExistenceRequirement::KeepAlive,
            );
            // if successful
            if let Ok(imbalance) = res {
                // perform refund
                let res = CurrencyOf::<T>::resolve_into_existing(&queued_refund.account, imbalance);
                match res {
                    // take money from refund account OK + refund account OK → event
                    Ok(_) => {
                        Self::deposit_event(Event::Refunded {
                            who: queued_refund.account,
                            identity: queued_refund.identity,
                            amount,
                        });
                    }
                    Err(imbalance) => {
                        // refund failed (for example account stopped existing) → handle dust
                        // give back to refund account (should not happen)
                        CurrencyOf::<T>::resolve_creating(&T::RefundAccount::get(), imbalance);
                        // if this event is observed, block should be examined carefully
                        Self::deposit_event(Event::RefundFailed(queued_refund.account));
                    }
                }
            } else {
                // could not withdraw refund account
                Self::deposit_event(Event::NoMoreCurrencyForRefund);
            }
        }

        /// Processes as many refunds as possible from the refund queue within the supplied weight limit.
        pub fn process_refund_queue(weight_limit: Weight) -> Weight {
            RefundQueue::<T>::mutate(|queue| {
                // The weight to process an empty queue
                let mut total_weight = <T as pallet::Config>::WeightInfo::on_process_refund_queue();
                // The weight to process one element without the actual try_refund weight
                let overhead =
                    <T as pallet::Config>::WeightInfo::on_process_refund_queue_elements(2)
                        .saturating_sub(
                            <T as pallet::Config>::WeightInfo::on_process_refund_queue_elements(1),
                        )
                        .saturating_sub(<T as pallet::Config>::WeightInfo::try_refund());

                // make sure that we have at least the time to handle one try_refund call
                if queue.is_empty() {
                    return total_weight;
                }

                while total_weight.any_lt(weight_limit.saturating_sub(
                    <T as pallet::Config>::WeightInfo::try_refund().saturating_add(overhead),
                )) {
                    let Some(queued_refund) = queue.pop() else {
                        break;
                    };
                    let consumed_weight = Self::try_refund(queued_refund);
                    total_weight = total_weight
                        .saturating_add(consumed_weight)
                        .saturating_add(overhead);
                }
                total_weight
            })
        }

        /// Spends the quota of an identity by deducting the specified `amount` from its quota balance.
        pub fn spend_quota(idty_id: IdtyId<T>, amount: BalanceOf<T>) -> BalanceOf<T> {
            IdtyQuota::<T>::mutate_exists(idty_id, |quota| {
                if let Some(quota) = quota {
                    Self::update_quota(quota);
                    Self::do_spend_quota(quota, amount)
                } else {
                    // error event if identity has no quota
                    Self::deposit_event(Event::NoQuotaForIdty(idty_id));
                    BalanceOf::<T>::zero()
                }
            })
        }

        /// Update the quota according to the growth rate, maximum value, and last use.
        fn update_quota(quota: &mut Quota<BlockNumberFor<T>, BalanceOf<T>>) {
            let current_block = frame_system::pallet::Pallet::<T>::block_number();
            let quota_growth = sp_runtime::Perbill::from_rational(
                current_block - quota.last_use,
                T::ReloadRate::get(),
            )
            .mul_floor(T::MaxQuota::get());
            // mutate quota
            quota.last_use = current_block;
            quota.amount = core::cmp::min(quota.amount + quota_growth, T::MaxQuota::get());
        }

        /// Spend a certain amount of quota and return the amount that was spent.
        fn do_spend_quota(
            quota: &mut Quota<BlockNumberFor<T>, BalanceOf<T>>,
            amount: BalanceOf<T>,
        ) -> BalanceOf<T> {
            let old_amount = quota.amount;
            // entire amount fit in remaining quota
            if amount <= old_amount {
                quota.amount -= amount;
                amount
            }
            // all quota are spent and only partial refund is possible
            else {
                quota.amount = BalanceOf::<T>::zero();
                old_amount
            }
        }
    }

    // GENESIS STUFF //
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub identities: Vec<IdtyId<T>>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                identities: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for idty in self.identities.iter() {
                IdtyQuota::<T>::insert(
                    idty,
                    Quota {
                        last_use: BlockNumberFor::<T>::zero(),
                        amount: BalanceOf::<T>::zero(),
                    },
                );
            }
        }
    }

    // HOOKS //
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // process refund queue if space left on block
        fn on_idle(_block: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            Self::process_refund_queue(remaining_weight)
        }
    }
}

/// Implementing the refund fee trait for the pallet.
impl<T: Config> RefundFee<T> for Pallet<T> {
    /// This implementation checks if the identity is eligible for a refund and queues the refund if so.
    fn request_refund(account: T::AccountId, identity: IdtyId<T>, amount: BalanceOf<T>) {
        if is_eligible_for_refund::<T>(identity) {
            Self::queue_refund(Refund {
                account,
                identity,
                amount,
            })
        }
    }
}

/// Checks if an identity is eligible for a refund.
///
/// This function returns `true` only if the identity exists and has a status of `Member`.
/// If the identity does not exist or has a different status, it returns `false`, and the refund request will not be processed.
///
fn is_eligible_for_refund<T: pallet_identity::Config>(idty_index: IdtyId<T>) -> bool {
    pallet_identity::Identities::<T>::get(idty_index).map_or_else(
        || false,
        |id| id.status == pallet_identity::IdtyStatus::Member,
    )
}

/// Implementing the on new membership event handler for the pallet.
impl<T: Config> sp_membership::traits::OnNewMembership<IdtyId<T>> for Pallet<T> {
    /// This implementation initializes the identity quota for the newly created identity.
    fn on_created(idty_index: &IdtyId<T>) {
        IdtyQuota::<T>::insert(
            idty_index,
            Quota {
                last_use: frame_system::pallet::Pallet::<T>::block_number(),
                amount: BalanceOf::<T>::zero(),
            },
        );
    }

    fn on_renewed(_idty_index: &IdtyId<T>) {}
}

/// Implementing the on remove identity event handler for the pallet.
impl<T: Config> sp_membership::traits::OnRemoveMembership<IdtyId<T>> for Pallet<T> {
    /// This implementation removes the identity quota associated with the removed identity.
    fn on_removed(idty_id: &IdtyId<T>) -> Weight {
        let mut weight = Weight::zero();
        let mut add_db_reads_writes = |reads, writes| {
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(reads, writes));
        };

        IdtyQuota::<T>::remove(idty_id);
        add_db_reads_writes(1, 1);
        weight
    }
}
