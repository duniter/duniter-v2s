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
    use scale_info::TypeInfo;

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    // STORAGE //

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // A value placed in storage that represents the current version of the Balances storage.
    // This value is used by the `on_runtime_upgrade` logic to determine whether we run
    // storage migration logic. This should match directly with the semantic versions of the Rust crate.
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub enum Releases {
        V1_0_0,
    }
    impl Default for Releases {
        fn default() -> Self {
            Releases::V1_0_0
        }
    }

    /// Storage version of the pallet.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ud_accounts)]
    pub type UdAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ud_accounts_count)]
    pub(super) type UdAccountsCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub(super) type ToBeRemoved<T: Config> = StorageValue<_, Vec<u32>, ValueQuery>;

    // GENESIS //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub ud_accounts: sp_std::collections::btree_map::BTreeMap<T::AccountId, u32>,
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
            <StorageVersion<T>>::put(Releases::V1_0_0);
            <UdAccountsCounter<T>>::put(self.ud_accounts.len() as u64);
            for (account, index) in &self.ud_accounts {
                <UdAccounts<T>>::insert(account, index);
            }
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn account_list() -> Vec<T::AccountId> {
            let mut to_be_removed = ToBeRemoved::<T>::take();
            to_be_removed.sort_unstable();

            let mut accounts_to_pass = Vec::new();
            let mut accounts_to_remove = Vec::new();
            <UdAccounts<T>>::iter().for_each(|(k, v)| {
                if to_be_removed.binary_search(&v).is_ok() {
                    accounts_to_remove.push(k);
                } else {
                    accounts_to_pass.push(k);
                }
            });
            for account in accounts_to_remove {
                UdAccounts::<T>::remove(account);
            }

            accounts_to_pass
        }
        pub fn replace_account(
            old_account_opt: Option<T::AccountId>,
            new_account_opt: Option<T::AccountId>,
            index: u32,
        ) -> Weight {
            if let Some(old_account) = old_account_opt {
                if let Some(new_account) = new_account_opt {
                    Self::replace_account_inner(old_account, new_account, index)
                } else {
                    Self::del_account(old_account)
                }
            } else if let Some(new_account) = new_account_opt {
                Self::add_account(new_account, index)
            } else {
                0
            }
        }
        pub fn remove_account(account_index: u32) -> Weight {
            ToBeRemoved::<T>::append(account_index);
            UdAccountsCounter::<T>::mutate(|counter| counter.saturating_sub(1));
            0
        }
        fn replace_account_inner(
            old_account: T::AccountId,
            new_account: T::AccountId,
            index: u32,
        ) -> Weight {
            if <UdAccounts<T>>::contains_key(&old_account) {
                if !<UdAccounts<T>>::contains_key(&new_account) {
                    <UdAccounts<T>>::remove(&old_account);
                    <UdAccounts<T>>::insert(&new_account, index);
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
        fn add_account(account: T::AccountId, index: u32) -> Weight {
            if !<UdAccounts<T>>::contains_key(&account) {
                <UdAccounts<T>>::insert(&account, index);
                UdAccountsCounter::<T>::mutate(|counter| counter.saturating_add(1));
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

                UdAccountsCounter::<T>::mutate(|counter| counter.saturating_sub(1));
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
