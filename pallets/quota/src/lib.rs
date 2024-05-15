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

pub mod traits;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use crate::traits::*;
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::Zero;
use sp_std::{fmt::Debug, vec::Vec};
pub use weights::WeightInfo;

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
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// number of blocks in which max quota is replenished
        type ReloadRate: Get<BlockNumberFor<Self>>;
        /// maximum amount of quota an identity can get
        type MaxQuota: Get<BalanceOf<Self>>;
        /// Account used to refund fee
        #[pallet::constant]
        type RefundAccount: Get<Self::AccountId>;
        /// Weight
        type WeightInfo: WeightInfo;
    }

    // TYPES //
    #[derive(Encode, Decode, Clone, TypeInfo, Debug, PartialEq, MaxEncodedLen)]
    pub struct Refund<AccountId, IdtyId, Balance> {
        /// account to refund
        pub account: AccountId,
        /// identity to use quota
        pub identity: IdtyId,
        /// amount of refund
        pub amount: Balance,
    }

    #[derive(Encode, Decode, Clone, TypeInfo, Debug, PartialEq, MaxEncodedLen)]
    pub struct Quota<BlockNumber, Balance> {
        /// block number of last quota use
        pub last_use: BlockNumber,
        /// amount of remaining quota
        pub amount: Balance,
    }

    // STORAGE //
    /// maps identity index to quota
    #[pallet::storage]
    #[pallet::getter(fn quota)]
    pub type IdtyQuota<T: Config> =
        StorageMap<_, Twox64Concat, IdtyId<T>, Quota<BlockNumberFor<T>, BalanceOf<T>>, OptionQuery>;

    /// fees waiting for refund
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

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        /// add a new refund to the queue
        pub fn queue_refund(refund: Refund<T::AccountId, IdtyId<T>, BalanceOf<T>>) {
            if RefundQueue::<T>::mutate(|v| v.try_push(refund)).is_err() {
                Self::deposit_event(Event::RefundQueueFull);
            }
        }

        /// try to refund using quota if available
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

        /// do refund a non-null amount
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

        /// perform as many refunds as possible within the supplied weight limit
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

        /// spend quota of identity
        pub fn spend_quota(idty_id: IdtyId<T>, amount: BalanceOf<T>) -> BalanceOf<T> {
            IdtyQuota::<T>::mutate_exists(idty_id, |quota| {
                if let Some(ref mut quota) = quota {
                    Self::update_quota(quota);
                    Self::do_spend_quota(quota, amount)
                } else {
                    // error event if identity has no quota
                    Self::deposit_event(Event::NoQuotaForIdty(idty_id));
                    BalanceOf::<T>::zero()
                }
            })
        }

        /// update quota according to the growth rate, max value, and last use
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

        /// spend a certain amount of quota and return what was spent
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
/// This function returns `true` for all identities, regardless of their status.
/// If the identity has no quotas or has been deleted, the refund request is still queued,
/// but when handled, no refund will be issued (and `NoQuotaForIdty` may be raised).
fn is_eligible_for_refund<T: pallet_identity::Config>(_identity: IdtyId<T>) -> bool {
    true
}

/// Implementing the on new identity event handler for the pallet.
impl<T: Config> pallet_identity::traits::OnNewIdty<T> for Pallet<T> {
    /// This implementation initializes the identity quota for the newly created identity.
    fn on_created(idty_index: &IdtyId<T>, _creator: &T::IdtyIndex) {
        IdtyQuota::<T>::insert(
            idty_index,
            Quota {
                last_use: frame_system::pallet::Pallet::<T>::block_number(),
                amount: BalanceOf::<T>::zero(),
            },
        );
    }
}

/// Implementing the on remove identity event handler for the pallet.
impl<T: Config> pallet_identity::traits::OnRemoveIdty<T> for Pallet<T> {
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

    /// This implementation removes the identity quota associated with the removed identity.
    fn on_revoked(idty_id: &IdtyId<T>) -> Weight {
        Self::on_removed(idty_id)
    }
}
