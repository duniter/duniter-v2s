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
#![allow(clippy::unused_unit)]

pub use pallet::*;

/*#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
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
    pub trait Config: frame_system::Config {}

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn ud_accounts)]
    pub type UdAccounts<T: Config> =
        CountedStorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

    // GENESIS //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub ud_accounts: sp_std::collections::btree_set::BTreeSet<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                ud_accounts: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for account in &self.ud_accounts {
                <UdAccounts<T>>::insert(account, ());
            }
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn accounts_len() -> u32 {
            <UdAccounts<T>>::count()
        }
        pub fn accounts_list() -> Vec<T::AccountId> {
            <UdAccounts<T>>::iter_keys().collect()
        }
        pub fn replace_account(
            old_account_opt: Option<T::AccountId>,
            new_account_opt: Option<T::AccountId>,
        ) -> Weight {
            if let Some(old_account) = old_account_opt {
                if let Some(new_account) = new_account_opt {
                    Self::replace_account_inner(old_account, new_account)
                } else {
                    Self::del_account(old_account)
                }
            } else if let Some(new_account) = new_account_opt {
                Self::add_account(new_account)
            } else {
                0
            }
        }
        pub fn remove_account(account_id: T::AccountId) -> Weight {
            Self::del_account(account_id)
        }
        fn replace_account_inner(old_account: T::AccountId, new_account: T::AccountId) -> Weight {
            if <UdAccounts<T>>::contains_key(&old_account) {
                if !<UdAccounts<T>>::contains_key(&new_account) {
                    <UdAccounts<T>>::remove(&old_account);
                    <UdAccounts<T>>::insert(&new_account, ());
                } else {
                    frame_support::runtime_print!(
                        "ERROR: replace_account(): new_account {:?} already added",
                        new_account
                    );
                }
            } else {
                frame_support::runtime_print!(
                    "ERROR: replace_account(): old_account {:?} already deleted",
                    old_account
                );
            }
            0
        }
        fn add_account(account: T::AccountId) -> Weight {
            if !<UdAccounts<T>>::contains_key(&account) {
                <UdAccounts<T>>::insert(&account, ());
            } else {
                frame_support::runtime_print!(
                    "ERROR: add_account(): account {:?} already added",
                    account
                );
            }
            0
        }
        fn del_account(account: T::AccountId) -> Weight {
            if <UdAccounts<T>>::contains_key(&account) {
                <UdAccounts<T>>::remove(&account);
            } else {
                frame_support::runtime_print!(
                    "ERROR: del_account(): account {:?} already deleted",
                    account
                );
            }
            0
        }
    }
}
