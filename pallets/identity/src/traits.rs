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

use crate::*;
use frame_support::pallet_prelude::*;

/// Trait defining operations for checking if identity-related calls are allowed.
pub trait CheckIdtyCallAllowed<T: Config> {
    /// Check if creating an identity is allowed.
    fn check_create_identity(creator: T::IdtyIndex) -> Result<(), DispatchError>;
}

impl<T: Config> CheckIdtyCallAllowed<T> for () {
    fn check_create_identity(_creator: T::IdtyIndex) -> Result<(), DispatchError> {
        Ok(())
    }
}

/// Trait to check the worthiness of an account.
pub trait CheckAccountWorthiness<T: Config> {
    /// Check the worthiness of an account.
    fn check_account_worthiness(account: &T::AccountId) -> Result<(), DispatchError>;
    /// Set an account as worthy. Only available for runtime benchmarks.
    #[cfg(feature = "runtime-benchmarks")]
    fn set_worthy(account: &T::AccountId);
}

impl<T: Config> CheckAccountWorthiness<T> for () {
    fn check_account_worthiness(_account: &T::AccountId) -> Result<(), DispatchError> {
        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn set_worthy(_account: &T::AccountId) {}
}

/// Trait defining operations for validating identity names.
pub trait IdtyNameValidator {
    /// Validate an identity name.
    fn validate(idty_name: &IdtyName) -> bool;
}

/// Trait defining behavior for handling new identities creation.
pub trait OnNewIdty<T: Config> {
    /// Called when a new identity is created.
    fn on_created(idty_index: &T::IdtyIndex, creator: &T::IdtyIndex);
}

/// Trait defining behavior for handling removed identities.
/// As the weight accounting can be complicated it should be done
/// at the handler level.
pub trait OnRemoveIdty<T: Config> {
    /// Called when an identity is removed.
    fn on_removed(idty_index: &T::IdtyIndex) -> Weight;
    /// Called when an identity is revoked.
    fn on_revoked(idty_index: &T::IdtyIndex) -> Weight;
}

impl<T: Config> OnNewIdty<T> for () {
    fn on_created(_idty_index: &T::IdtyIndex, _creator: &T::IdtyIndex) {}
}

impl<T: Config> OnRemoveIdty<T> for () {
    fn on_removed(_idty_index: &T::IdtyIndex) -> Weight {
        Weight::zero()
    }

    fn on_revoked(_idty_index: &T::IdtyIndex) -> Weight {
        Weight::zero()
    }
}

/// Trait defining operations for linking identities to accounts.
pub trait LinkIdty<AccountId, IdtyIndex> {
    /// Links an identity to an account.
    fn link_identity(account_id: &AccountId, idty_index: IdtyIndex) -> Result<(), DispatchError>;
}

impl<AccountId, IdtyIndex> LinkIdty<AccountId, IdtyIndex> for () {
    fn link_identity(_: &AccountId, _: IdtyIndex) -> Result<(), DispatchError> {
        Ok(())
    }
}

/// A trait for handling identity owner key changes.
pub trait KeyChange<T: Config> {
    fn on_changed(id: T::IdtyIndex, account_id: T::AccountId) -> Result<(), DispatchError>;
}

impl<T: Config> KeyChange<T> for () {
    /// Called when an identity's owner key has changed.
    fn on_changed(_id: T::IdtyIndex, _account_id: T::AccountId) -> Result<(), DispatchError> {
        Ok(())
    }
}
