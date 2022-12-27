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
mod check_nonce;
#[cfg(test)]
mod mock;
mod types;

pub use check_nonce::CheckNonce;
pub use pallet::*;
pub use types::*;

use frame_support::pallet_prelude::*;
use frame_support::traits::{
    Currency, ExistenceRequirement, Imbalance, IsSubType, WithdrawReasons,
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::OnChargeTransaction;
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup, Zero};
use sp_std::convert::TryInto;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        type Currency: Currency<Self::AccountId>;
        type InnerOnChargeTransaction: OnChargeTransaction<Self>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn oneshot_account)]
    pub type OneshotAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        <T::Currency as Currency<T::AccountId>>::Balance,
        OptionQuery,
    >;

    // EVENTS //

    #[allow(clippy::type_complexity)]
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OneshotAccountCreated {
            account: T::AccountId,
            balance: <T::Currency as Currency<T::AccountId>>::Balance,
            creator: T::AccountId,
        },
        OneshotAccountConsumed {
            account: T::AccountId,
            dest1: (
                T::AccountId,
                <T::Currency as Currency<T::AccountId>>::Balance,
            ),
            dest2: Option<(
                T::AccountId,
                <T::Currency as Currency<T::AccountId>>::Balance,
            )>,
        },
        Withdraw {
            account: T::AccountId,
            balance: <T::Currency as Currency<T::AccountId>>::Balance,
        },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Block height is in the future
        BlockHeightInFuture,
        /// Block height is too old
        BlockHeightTooOld,
        /// Destination account does not exist
        DestAccountNotExist,
        /// Destination account has balance less than existential deposit
        ExistentialDeposit,
        /// Source account has insufficient balance
        InsufficientBalance,
        /// Destination oneshot account already exists
        OneshotAccountAlreadyCreated,
        /// Source oneshot account does not exist
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
        #[pallet::weight(500_000_000)]
        pub fn create_oneshot_account(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;

            ensure!(
                value >= <T::Currency as Currency<T::AccountId>>::minimum_balance(),
                Error::<T>::ExistentialDeposit
            );
            ensure!(
                OneshotAccounts::<T>::get(&dest).is_none(),
                Error::<T>::OneshotAccountAlreadyCreated
            );

            <T::Currency as Currency<T::AccountId>>::withdraw(
                &transactor,
                value,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::KeepAlive,
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
        #[pallet::weight(500_000_000)]
        pub fn consume_oneshot_account(
            origin: OriginFor<T>,
            block_height: T::BlockNumber,
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
            } else {
                <T::Currency as Currency<T::AccountId>>::deposit_into_existing(&dest, value)?;
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
        #[pallet::weight(500_000_000)]
        pub fn consume_oneshot_account_with_remaining(
            origin: OriginFor<T>,
            block_height: T::BlockNumber,
            dest: Account<<T::Lookup as StaticLookup>::Source>,
            remaining_to: Account<<T::Lookup as StaticLookup>::Source>,
            #[pallet::compact] balance: <T::Currency as Currency<T::AccountId>>::Balance,
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
                    balance1 >= <T::Currency as Currency<T::AccountId>>::minimum_balance(),
                    Error::<T>::ExistentialDeposit
                );
            } else {
                ensure!(
                    !<T::Currency as Currency<T::AccountId>>::free_balance(&dest1).is_zero(),
                    Error::<T>::DestAccountNotExist
                );
            }
            if dest2_is_oneshot {
                ensure!(
                    OneshotAccounts::<T>::get(&dest2).is_none(),
                    Error::<T>::OneshotAccountAlreadyCreated
                );
                ensure!(
                    balance2 >= <T::Currency as Currency<T::AccountId>>::minimum_balance(),
                    Error::<T>::ExistentialDeposit
                );
                OneshotAccounts::<T>::insert(&dest2, balance2);
                Self::deposit_event(Event::OneshotAccountCreated {
                    account: dest2.clone(),
                    balance: balance2,
                    creator: transactor.clone(),
                });
            } else {
                <T::Currency as Currency<T::AccountId>>::deposit_into_existing(&dest2, balance2)?;
            }
            if dest1_is_oneshot {
                OneshotAccounts::<T>::insert(&dest1, balance1);
                Self::deposit_event(Event::OneshotAccountCreated {
                    account: dest1.clone(),
                    balance: balance1,
                    creator: transactor.clone(),
                });
            } else {
                <T::Currency as Currency<T::AccountId>>::deposit_into_existing(&dest1, balance1)?;
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
        Balance = <T::Currency as Currency<T::AccountId>>::Balance,
        LiquidityInfo = Option<<T::Currency as Currency<T::AccountId>>::NegativeImbalance>,
    >,
{
    type Balance = <T::Currency as Currency<T::AccountId>>::Balance;
    type LiquidityInfo = Option<<T::Currency as Currency<T::AccountId>>::NegativeImbalance>;
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
                    // TODO
                    return Ok(Some(
                        <T::Currency as Currency<T::AccountId>>::NegativeImbalance::zero(),
                    ));
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
}
