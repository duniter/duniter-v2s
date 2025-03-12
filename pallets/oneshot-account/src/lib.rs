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

//! # Duniter Oneshot Account Pallet
//!
//! Duniter Oneshot Account Pallet introduces lightweight accounts that do not utilize `AccountInfo`, including fields like nonce, consumers, providers, sufficients, free, reserved. These accounts are designed for single-use scenarios, aiming to reduce transaction weight and associated fees. The primary use cases include anonymous transactions and physical support scenarios where lightweight and disposable accounts are beneficial.

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod check_nonce;
#[cfg(test)]
mod mock;
mod types;
pub mod weights;

pub use check_nonce::CheckNonce;
pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::{
    pallet_prelude::*,
    traits::{
        fungible,
        fungible::{Balanced, Credit, Inspect},
        tokens::{Fortitude, Precision, Preservation},
        Imbalance, IsSubType,
    },
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::OnChargeTransaction;
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup, Zero};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as fungible::Inspect<AccountIdOf<T>>>::Balance;

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The currency type.
        type Currency: fungible::Balanced<Self::AccountId> + fungible::Mutate<Self::AccountId>;

        /// A handler for charging transactions.
        type InnerOnChargeTransaction: OnChargeTransaction<Self>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet.
        type WeightInfo: WeightInfo;
    }

    // STORAGE //

    /// The balance for each oneshot account.
    #[pallet::storage]
    #[pallet::getter(fn oneshot_account)]
    pub type OneshotAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    // EVENTS //

    #[allow(clippy::type_complexity)]
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A oneshot account was created.
        OneshotAccountCreated {
            account: T::AccountId,
            balance: BalanceOf<T>,
            creator: T::AccountId,
        },
        /// A oneshot account was consumed.
        OneshotAccountConsumed {
            account: T::AccountId,
            dest1: (T::AccountId, BalanceOf<T>),
            dest2: Option<(T::AccountId, BalanceOf<T>)>,
        },
        /// A withdrawal was executed on a oneshot account.
        Withdraw {
            account: T::AccountId,
            balance: BalanceOf<T>,
        },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Block height is in the future.
        BlockHeightInFuture,
        /// Block height is too old.
        BlockHeightTooOld,
        /// Destination account does not exist.
        DestAccountNotExist,
        /// Destination account has a balance less than the existential deposit.
        ExistentialDeposit,
        /// Source account has insufficient balance.
        InsufficientBalance,
        /// Destination oneshot account already exists.
        OneshotAccountAlreadyCreated,
        /// Source oneshot account does not exist.
        OneshotAccountNotExist,
    }

    // CALLS //
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create an account that can only be consumed once
        ///
        /// - `dest`: The oneshot account to be created.
        /// - `balance`: The balance to be transfered to this oneshot account.
        ///
        /// Origin account is kept alive.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_oneshot_account())]
        pub fn create_oneshot_account(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;

            ensure!(
                value >= T::Currency::minimum_balance(),
                Error::<T>::ExistentialDeposit
            );
            ensure!(
                OneshotAccounts::<T>::get(&dest).is_none(),
                Error::<T>::OneshotAccountAlreadyCreated
            );

            let _ = T::Currency::withdraw(
                &transactor,
                value,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )?;
            OneshotAccounts::<T>::insert(&dest, value);
            Self::deposit_event(Event::OneshotAccountCreated {
                account: dest,
                balance: value,
                creator: transactor,
            });

            Ok(())
        }

        /// Consume a oneshot account and transfer its balance to an account
        ///
        /// - `block_height`: Must be a recent block number. The limit is `BlockHashCount` in the past. (this is to prevent replay attacks)
        /// - `dest`: The destination account.
        /// - `dest_is_oneshot`: If set to `true`, then a oneshot account is created at `dest`. Else, `dest` has to be an existing account.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::consume_oneshot_account())]
        pub fn consume_oneshot_account(
            origin: OriginFor<T>,
            block_height: BlockNumberFor<T>,
            dest: Account<<T::Lookup as StaticLookup>::Source>,
        ) -> DispatchResult {
            let transactor = ensure_signed(origin)?;

            let (dest, dest_is_oneshot) = match dest {
                Account::Normal(account) => (account, false),
                Account::Oneshot(account) => (account, true),
            };
            let dest = T::Lookup::lookup(dest)?;

            let value = OneshotAccounts::<T>::take(&transactor)
                .ok_or(Error::<T>::OneshotAccountNotExist)?;

            ensure!(
                block_height <= frame_system::Pallet::<T>::block_number(),
                Error::<T>::BlockHeightInFuture
            );
            ensure!(
                frame_system::pallet::BlockHash::<T>::contains_key(block_height),
                Error::<T>::BlockHeightTooOld
            );
            if dest_is_oneshot {
                ensure!(
                    OneshotAccounts::<T>::get(&dest).is_none(),
                    Error::<T>::OneshotAccountAlreadyCreated
                );
                OneshotAccounts::<T>::insert(&dest, value);
                Self::deposit_event(Event::OneshotAccountCreated {
                    account: dest.clone(),
                    balance: value,
                    creator: transactor.clone(),
                });
            } else if frame_system::Pallet::<T>::providers(&dest) > 0 {
                let _ = T::Currency::deposit(&dest, value, Precision::Exact)?;
            }
            OneshotAccounts::<T>::remove(&transactor);
            Self::deposit_event(Event::OneshotAccountConsumed {
                account: transactor,
                dest1: (dest, value),
                dest2: None,
            });

            Ok(())
        }

        /// Consume a oneshot account then transfer some amount to an account,
        /// and the remaining amount to another account.
        ///
        /// - `block_height`: Must be a recent block number.
        ///   The limit is `BlockHashCount` in the past. (this is to prevent replay attacks)
        /// - `dest`: The destination account.
        /// - `dest_is_oneshot`: If set to `true`, then a oneshot account is created at `dest`. Else, `dest` has to be an existing account.
        /// - `dest2`: The second destination account.
        /// - `dest2_is_oneshot`: If set to `true`, then a oneshot account is created at `dest2`. Else, `dest2` has to be an existing account.
        /// - `balance1`: The amount transfered to `dest`, the leftover being transfered to `dest2`.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::consume_oneshot_account_with_remaining())]
        pub fn consume_oneshot_account_with_remaining(
            origin: OriginFor<T>,
            block_height: BlockNumberFor<T>,
            dest: Account<<T::Lookup as StaticLookup>::Source>,
            remaining_to: Account<<T::Lookup as StaticLookup>::Source>,
            #[pallet::compact] balance: BalanceOf<T>,
        ) -> DispatchResult {
            let transactor = ensure_signed(origin)?;

            let (dest1, dest1_is_oneshot) = match dest {
                Account::Normal(account) => (account, false),
                Account::Oneshot(account) => (account, true),
            };
            let dest1 = T::Lookup::lookup(dest1)?;
            let (dest2, dest2_is_oneshot) = match remaining_to {
                Account::Normal(account) => (account, false),
                Account::Oneshot(account) => (account, true),
            };
            let dest2 = T::Lookup::lookup(dest2)?;

            let value = OneshotAccounts::<T>::take(&transactor)
                .ok_or(Error::<T>::OneshotAccountNotExist)?;

            let balance1 = balance;
            ensure!(value > balance1, Error::<T>::InsufficientBalance);
            let balance2 = value.saturating_sub(balance1);
            ensure!(
                block_height <= frame_system::Pallet::<T>::block_number(),
                Error::<T>::BlockHeightInFuture
            );
            ensure!(
                frame_system::pallet::BlockHash::<T>::contains_key(block_height),
                Error::<T>::BlockHeightTooOld
            );
            if dest1_is_oneshot {
                ensure!(
                    OneshotAccounts::<T>::get(&dest1).is_none(),
                    Error::<T>::OneshotAccountAlreadyCreated
                );
                ensure!(
                    balance1 >= T::Currency::minimum_balance(),
                    Error::<T>::ExistentialDeposit
                );
            } else {
                ensure!(
                    !T::Currency::balance(&dest1).is_zero(),
                    Error::<T>::DestAccountNotExist
                );
            }
            if dest2_is_oneshot {
                ensure!(
                    OneshotAccounts::<T>::get(&dest2).is_none(),
                    Error::<T>::OneshotAccountAlreadyCreated
                );
                ensure!(
                    balance2 >= T::Currency::minimum_balance(),
                    Error::<T>::ExistentialDeposit
                );
                OneshotAccounts::<T>::insert(&dest2, balance2);
                Self::deposit_event(Event::OneshotAccountCreated {
                    account: dest2.clone(),
                    balance: balance2,
                    creator: transactor.clone(),
                });
            } else if frame_system::Pallet::<T>::providers(&dest2) > 0 {
                let _ = T::Currency::deposit(&dest2, balance2, Precision::Exact)?;
            }
            if dest1_is_oneshot {
                OneshotAccounts::<T>::insert(&dest1, balance1);
                Self::deposit_event(Event::OneshotAccountCreated {
                    account: dest1.clone(),
                    balance: balance1,
                    creator: transactor.clone(),
                });
            } else if frame_system::Pallet::<T>::providers(&dest1) > 0 {
                let _ = T::Currency::deposit(&dest1, balance1, Precision::Exact)?;
            }
            OneshotAccounts::<T>::remove(&transactor);
            Self::deposit_event(Event::OneshotAccountConsumed {
                account: transactor,
                dest1: (dest1, balance1),
                dest2: Some((dest2, balance2)),
            });

            Ok(())
        }
    }
}

impl<T: Config> OnChargeTransaction<T> for Pallet<T>
where
    T::RuntimeCall: IsSubType<Call<T>>,
    T::InnerOnChargeTransaction: OnChargeTransaction<
        T,
        Balance = BalanceOf<T>,
        LiquidityInfo = Option<Credit<T::AccountId, T::Currency>>,
    >,
{
    type Balance = BalanceOf<T>;
    type LiquidityInfo = Option<Credit<T::AccountId, T::Currency>>;

    fn can_withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        T::InnerOnChargeTransaction::can_withdraw_fee(who, call, dispatch_info, fee, tip)
    }

    fn withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        if let Some(
            Call::consume_oneshot_account { .. }
            | Call::consume_oneshot_account_with_remaining { .. },
        ) = call.is_sub_type()
        {
            if fee.is_zero() {
                return Ok(None);
            }

            if let Some(balance) = OneshotAccounts::<T>::get(who) {
                if balance >= fee {
                    OneshotAccounts::<T>::insert(who, balance.saturating_sub(fee));
                    Self::deposit_event(Event::Withdraw {
                        account: who.clone(),
                        balance: fee,
                    });
                    return Ok(Some(Imbalance::zero()));
                }
            }
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::Payment,
            ))
        } else {
            T::InnerOnChargeTransaction::withdraw_fee(who, call, dispatch_info, fee, tip)
        }
    }

    fn correct_and_deposit_fee(
        who: &T::AccountId,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        T::InnerOnChargeTransaction::correct_and_deposit_fee(
            who,
            dispatch_info,
            post_info,
            corrected_fee,
            tip,
            already_withdrawn,
        )
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &T::AccountId, amount: Self::Balance) {
        T::InnerOnChargeTransaction::endow_account(who, amount);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance {
        T::InnerOnChargeTransaction::minimum_balance()
    }
}
