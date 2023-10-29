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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub mod weights;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::pallet_prelude::*;
use frame_support::traits::{OnUnbalanced, StoredMap};
use frame_system::pallet_prelude::*;
use pallet_provide_randomness::RequestId;
use sp_core::H256;
use sp_runtime::traits::{Convert, Saturating, Zero};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, ExistenceRequirement, StorageVersion};

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config<AccountData = AccountData<Self::Balance>>
        + pallet_balances::Config
        + pallet_provide_randomness::Config<Currency = pallet_balances::Pallet<Self>>
        + pallet_treasury::Config<Currency = pallet_balances::Pallet<Self>>
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
        pub accounts:
            sp_std::collections::btree_map::BTreeMap<T::AccountId, GenesisAccountData<T::Balance>>,
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
            // Classic accounts
            for (
                account_id,
                GenesisAccountData {
                    random_id,
                    balance,
                    is_identity,
                },
            ) in &self.accounts
            {
                assert!(!balance.is_zero() || *is_identity);
                frame_system::Account::<T>::mutate(account_id, |account| {
                    account.data.random_id = Some(*random_id);
                    if !balance.is_zero() {
                        account.data.free = *balance;
                        account.providers = 1;
                    }
                });
            }
        }
    }

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Force the destruction of an account because its free balance is insufficient to pay
        /// the account creation price.
        /// [who, balance]
        ForceDestroy {
            who: T::AccountId,
            balance: T::Balance,
        },
        /// Random id assigned
        /// [account_id, random_id]
        RandomIdAssigned { who: T::AccountId, random_id: H256 },
    }

    // HOOKS //
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
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

impl<T, AccountId, Balance>
    frame_support::traits::StoredMap<AccountId, pallet_balances::AccountData<Balance>> for Pallet<T>
where
    AccountId: Parameter
        + Member
        + MaybeSerializeDeserialize
        + core::fmt::Debug
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
        + core::fmt::Debug
        + codec::MaxEncodedLen
        + scale_info::TypeInfo,
    T: Config
        + frame_system::Config<AccountId = AccountId, AccountData = AccountData<Balance>>
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
        let was_providing = account.data.was_providing();
        let mut some_data = if was_providing {
            Some(account.data.into())
        } else {
            None
        };
        let result = f(&mut some_data)?;
        let is_providing = some_data.is_some();
        if !was_providing && is_providing {
            frame_system::Pallet::<T>::inc_providers(account_id);
            PendingNewAccounts::<T>::insert(account_id, ());
        } else if was_providing && !is_providing {
            match frame_system::Pallet::<T>::dec_providers(account_id)? {
                frame_system::DecRefStatus::Reaped => return Ok(result),
                frame_system::DecRefStatus::Exists => {
                    // Update value as normal...
                }
            }
        } else if !was_providing && !is_providing {
            return Ok(result);
        }
        frame_system::Account::<T>::mutate(account_id, |a| {
            a.data.set_balances(some_data.unwrap_or_default())
        });
        Ok(result)
    }
}
