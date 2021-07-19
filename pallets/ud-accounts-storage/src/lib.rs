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
    use sp_std::collections::btree_set::BTreeSet;

    use super::*;
    use frame_support::pallet_prelude::*;

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
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
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
    pub type UdAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ud_accounts_count)]
    pub(super) type UdAccountsCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    // GENESIS //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub ud_accounts: BTreeSet<T::AccountId>,
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
            for account in &self.ud_accounts {
                <UdAccounts<T>>::insert(account, ());
            }
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn account_list() -> Vec<T::AccountId> {
            <UdAccounts<T>>::iter().map(|(k, _v)| k).collect()
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
                if let Ok(counter) = <UdAccountsCounter<T>>::try_get() {
                    <UdAccountsCounter<T>>::put(counter.saturating_add(1));
                } else {
                    <UdAccountsCounter<T>>::put(1);
                }
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
                if let Ok(counter) = <UdAccountsCounter<T>>::try_get() {
                    <UdAccountsCounter<T>>::put(counter.saturating_sub(1));
                } else {
                    frame_support::runtime_print!(
                        "FATAL ERROR: del_account(): UdAccountsCounter is None!"
                    );
                }
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
