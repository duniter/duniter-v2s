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

// Note: refund queue mechanism is inspired from frame contract

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub mod weights;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement, StorageVersion};
use frame_support::traits::{OnUnbalanced, StoredMap};
use frame_system::pallet_prelude::*;
use pallet_provide_randomness::RequestId;
use pallet_quota::traits::RefundFee;
use pallet_transaction_payment::OnChargeTransaction;
use sp_core::H256;
use sp_runtime::traits::{Convert, DispatchInfoOf, PostDispatchInfoOf, Saturating};
use sp_std::fmt::Debug;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    pub type IdtyIdOf<T> = <T as pallet_identity::Config>::IdtyIndex;
    pub type CurrencyOf<T> = pallet_balances::Pallet<T>;
    pub type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config<AccountData = AccountData<Self::Balance, IdtyIdOf<Self>>>
        + pallet_balances::Config
        + pallet_provide_randomness::Config<Currency = pallet_balances::Pallet<Self>>
        + pallet_transaction_payment::Config
        + pallet_treasury::Config<Currency = pallet_balances::Pallet<Self>>
        + pallet_quota::Config
    {
        type AccountIdToSalt: Convert<Self::AccountId, [u8; 32]>;
        #[pallet::constant]
        type MaxNewAccountsPerBlock: Get<u32>;
        #[pallet::constant]
        type NewAccountPrice: Get<Self::Balance>;
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
        /// wrapped type
        type InnerOnChargeTransaction: OnChargeTransaction<Self>;
        /// type implementing refund behavior
        type Refund: pallet_quota::traits::RefundFee<Self>;
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn pending_random_id_assignments)]
    pub type PendingRandomIdAssignments<T: Config> =
        StorageMap<_, Twox64Concat, RequestId, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_new_accounts)]
    pub type PendingNewAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    // GENESIS STUFF //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub accounts: sp_std::collections::btree_map::BTreeMap<
            T::AccountId,
            GenesisAccountData<T::Balance, IdtyIdOf<T>>,
        >,
        pub treasury_balance: T::Balance,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                accounts: Default::default(),
                treasury_balance: T::ExistentialDeposit::get(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // Treasury
            frame_system::Account::<T>::mutate(
                pallet_treasury::Pallet::<T>::account_id(),
                |account| {
                    account.data.random_id = None;
                    account.data.free = self.treasury_balance;
                    account.providers = 1;
                },
            );

            // ensure no duplicate
            let endowed_accounts = self
                .accounts
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>();

            assert!(
                endowed_accounts.len() == self.accounts.len(),
                "duplicate balances in genesis."
            );

            // Classic accounts
            for (
                account_id,
                GenesisAccountData {
                    random_id,
                    balance,
                    idty_id,
                },
            ) in &self.accounts
            {
                // if the balance is below existential deposit, the account must be an identity
                assert!(balance >= &T::ExistentialDeposit::get() || idty_id.is_some());
                // mutate account
                frame_system::Account::<T>::mutate(account_id, |account| {
                    account.data.random_id = Some(*random_id);
                    account.data.free = *balance;
                    if idty_id.is_some() {
                        account.data.linked_idty = *idty_id;
                    }
                    if balance >= &T::ExistentialDeposit::get() {
                        // accounts above existential deposit self-provide
                        account.providers = 1;
                    }
                    // WARN (disabled) all genesis accounts provide for themselves whether they have existential deposit or not
                    // this is needed to migrate Ğ1 data where identities with zero Ğ1 can exist
                    // account.providers = 1;
                });
            }
        }
    }

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Forced destruction of an account due to insufficient free balance to cover the account creation price.
        ForceDestroy {
            who: T::AccountId,
            balance: T::Balance,
        },
        /// A random ID has been assigned to the account.
        RandomIdAssigned { who: T::AccountId, random_id: H256 },
        /// account linked to identity
        AccountLinked {
            who: T::AccountId,
            identity: IdtyIdOf<T>,
        },
        /// The account was unlinked from its identity.
        AccountUnlinked(T::AccountId),
    }

    // CALLS //
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// unlink the identity associated with the account
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::unlink_identity())]
        pub fn unlink_identity(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::do_unlink_identity(who);
            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        /// unlink account
        pub fn do_unlink_identity(account_id: T::AccountId) {
            // no-op if account already linked to nothing
            frame_system::Account::<T>::mutate(&account_id, |account| {
                if account.data.linked_idty.is_some() {
                    Self::deposit_event(Event::AccountUnlinked(account_id.clone()));
                }
                account.data.linked_idty = None;
            })
        }

        /// link account to identity
        pub fn do_link_identity(
            account_id: T::AccountId,
            idty_id: IdtyIdOf<T>,
        ) -> Result<(), DispatchError> {
            // Check that account exist
            ensure!(
                (frame_system::Account::<T>::get(&account_id).providers >= 1)
                    || (frame_system::Account::<T>::get(&account_id).sufficients >= 1),
                pallet_identity::Error::<T>::AccountNotExist
            );
            // no-op if identity does not change
            if frame_system::Account::<T>::get(&account_id)
                .data
                .linked_idty
                != Some(idty_id)
            {
                frame_system::Account::<T>::mutate(&account_id, |account| {
                    account.data.linked_idty = Some(idty_id);
                    Self::deposit_event(Event::AccountLinked {
                        who: account_id.clone(),
                        identity: idty_id,
                    });
                })
            }
            Ok(())
        }
    }

    // HOOKS //
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // on initialize, withdraw account creation tax
        fn on_initialize(_: T::BlockNumber) -> Weight {
            let mut total_weight = Weight::zero();
            for account_id in PendingNewAccounts::<T>::iter_keys()
                .drain()
                .take(T::MaxNewAccountsPerBlock::get() as usize)
            {
                if frame_system::Pallet::<T>::sufficients(&account_id) > 0 {
                    // If the account is self-sufficient, it is exempt from account creation fees
                    let request_id = pallet_provide_randomness::Pallet::<T>::force_request(
                        pallet_provide_randomness::RandomnessType::RandomnessFromTwoEpochsAgo,
                        H256(T::AccountIdToSalt::convert(account_id.clone())),
                    );
                    PendingRandomIdAssignments::<T>::insert(request_id, account_id);
                    total_weight += <T as pallet::Config>::WeightInfo::on_initialize_sufficient(
                        T::MaxNewAccountsPerBlock::get(),
                    );
                } else {
                    // If the account is not self-sufficient, it must pay the account creation fees
                    let account_data = frame_system::Pallet::<T>::get(&account_id);
                    let price = T::NewAccountPrice::get();
                    if account_data.free >= T::ExistentialDeposit::get() + price {
                        // The account can pay the new account price, we should:
                        // 1. Withdraw the "new account price" amount
                        // 2. Increment consumers to prevent the destruction of the account before
                        // the random id is assigned
                        // 3. Manage the funds collected
                        // 4. Submit random id generation request
                        // 5. Save the id of the random generation request.
                        let res = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::withdraw(
                            &account_id,
                            price,
                            frame_support::traits::WithdrawReasons::FEE,
                            ExistenceRequirement::KeepAlive,
                        );
                        debug_assert!(
                            res.is_ok(),
                            "Cannot fail because we checked that the free balance was sufficient"
                        );
                        if let Ok(imbalance) = res {
                            let res =
                                frame_system::Pallet::<T>::inc_consumers_without_limit(&account_id);
                            debug_assert!(
                                res.is_ok(),
                                "Cannot fail because any account with funds should have providers"
                            );
                            T::OnUnbalanced::on_unbalanced(imbalance);
                            let request_id = pallet_provide_randomness::Pallet::<T>::force_request(
                                pallet_provide_randomness::RandomnessType::RandomnessFromTwoEpochsAgo,
                                H256(T::AccountIdToSalt::convert(account_id.clone())),
                            );
                            PendingRandomIdAssignments::<T>::insert(request_id, account_id);
                            total_weight +=
                                <T as pallet::Config>::WeightInfo::on_initialize_with_balance(
                                    T::MaxNewAccountsPerBlock::get(),
                                );
                        }
                    } else {
                        // The charges could not be deducted, we must destroy the account
                        let balance_to_suppr =
                            account_data.free.saturating_add(account_data.reserved);
                        // Force account data supression
                        frame_system::Account::<T>::remove(&account_id);
                        Self::deposit_event(Event::ForceDestroy {
                            who: account_id,
                            balance: balance_to_suppr,
                        });
                        T::OnUnbalanced::on_unbalanced(pallet_balances::NegativeImbalance::new(
                            balance_to_suppr,
                        ));
                        total_weight += <T as pallet::Config>::WeightInfo::on_initialize_no_balance(
                            T::MaxNewAccountsPerBlock::get(),
                        );
                    }
                }
            }
            total_weight
        }
    }
}

// implement account linker
impl<T> pallet_identity::traits::LinkIdty<T::AccountId, IdtyIdOf<T>> for Pallet<T>
where
    T: Config,
{
    fn link_identity(account_id: T::AccountId, idty_id: IdtyIdOf<T>) -> Result<(), DispatchError> {
        Self::do_link_identity(account_id, idty_id)
    }
}

// implement on filled randomness
impl<T> pallet_provide_randomness::OnFilledRandomness for Pallet<T>
where
    T: Config,
{
    fn on_filled_randomness(request_id: RequestId, randomness: H256) -> Weight {
        if let Some(account_id) = PendingRandomIdAssignments::<T>::take(request_id) {
            frame_system::Account::<T>::mutate(&account_id, |account| {
                account.consumers = account.consumers.saturating_sub(1);
                account.data.random_id = Some(randomness);
            });
            Self::deposit_event(Event::RandomIdAssigned {
                who: account_id,
                random_id: randomness,
            });
            <T as pallet::Config>::WeightInfo::on_filled_randomness_pending()
        } else {
            <T as pallet::Config>::WeightInfo::on_filled_randomness_no_pending()
        }
    }
}

// implement accountdata storedmap
impl<T, AccountId, Balance>
    frame_support::traits::StoredMap<AccountId, pallet_balances::AccountData<Balance>> for Pallet<T>
where
    AccountId: Parameter
        + Member
        + MaybeSerializeDeserialize
        + Debug
        + sp_runtime::traits::MaybeDisplay
        + Ord
        + Into<[u8; 32]>
        + codec::MaxEncodedLen,
    Balance: Parameter
        + Member
        + sp_runtime::traits::AtLeast32BitUnsigned
        + codec::Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + codec::MaxEncodedLen
        + scale_info::TypeInfo,
    T: Config
        + frame_system::Config<AccountId = AccountId, AccountData = AccountData<Balance, IdtyIdOf<T>>>
        + pallet_balances::Config<Balance = Balance>
        + pallet_provide_randomness::Config,
{
    fn get(k: &AccountId) -> pallet_balances::AccountData<Balance> {
        frame_system::Account::<T>::get(k).data.into()
    }

    fn try_mutate_exists<R, E: From<sp_runtime::DispatchError>>(
        account_id: &AccountId,
        f: impl FnOnce(&mut Option<pallet_balances::AccountData<Balance>>) -> Result<R, E>,
    ) -> Result<R, E> {
        let account = frame_system::Account::<T>::get(account_id);
        let was_providing = !account.data.free.is_zero() || !account.data.reserved.is_zero();
        let mut some_data = if was_providing {
            Some(account.data.into())
        } else {
            None
        };
        let result = f(&mut some_data)?;
        let is_providing = some_data.is_some();
        match (was_providing, is_providing) {
            // the account has just been created, increment its provider and request a random_id
            (false, true) => {
                frame_system::Pallet::<T>::inc_providers(account_id);
                PendingNewAccounts::<T>::insert(account_id, ());
            }
            // the account was existing but is not anymore, decrement the provider
            (true, false) => {
                match frame_system::Pallet::<T>::dec_providers(account_id)? {
                    frame_system::DecRefStatus::Reaped => return Ok(result),
                    frame_system::DecRefStatus::Exists => {
                        // Update value as normal
                    }
                }
            }
            // mutation on unprovided account
            (false, false) => {
                return Ok(result);
            }
            // mutation on provided account
            (true, true) => {
                // Update value as normal
            }
        }
        // do mutate the account by setting the balances
        frame_system::Account::<T>::mutate(account_id, |a| {
            a.data.set_balances(some_data.unwrap_or_default())
        });
        Ok(result)
    }
}

// ------
// allows pay fees with quota instead of currency if available
impl<T: Config> OnChargeTransaction<T> for Pallet<T>
where
    T::InnerOnChargeTransaction: OnChargeTransaction<
        T,
        Balance = <CurrencyOf<T> as Currency<T::AccountId>>::Balance,
        LiquidityInfo = Option<<CurrencyOf<T> as Currency<T::AccountId>>::NegativeImbalance>,
    >,
{
    type Balance = BalanceOf<T>;
    type LiquidityInfo = Option<<CurrencyOf<T> as Currency<T::AccountId>>::NegativeImbalance>;

    fn withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        // does not change the withdraw fee step (still fallback to currency adapter or oneshot account)
        T::InnerOnChargeTransaction::withdraw_fee(who, call, dispatch_info, fee, tip)
    }

    fn correct_and_deposit_fee(
        who: &T::AccountId,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        // in any case, the default behavior is applied
        T::InnerOnChargeTransaction::correct_and_deposit_fee(
            who,
            dispatch_info,
            post_info,
            corrected_fee,
            tip,
            already_withdrawn,
        )?;
        // if account can be exonerated, add it to a refund queue
        let account_data = frame_system::Pallet::<T>::get(who);
        if let Some(idty_index) = account_data.linked_idty {
            T::Refund::request_refund(who.clone(), idty_index, corrected_fee.saturating_sub(tip));
        }
        Ok(())
    }
}
