// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use crate::*;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::MaybeSerializeDeserialize;
#[cfg(not(feature = "std"))]
use sp_std::fmt::Debug;
#[cfg(feature = "std")]
use std::fmt::Debug;

pub trait EnsureIdtyCallAllowed<T: Config> {
    fn create_identity(
        origin: T::Origin,
        creator: &T::IdtyDid,
        idty_did: &T::IdtyDid,
        idty_owner_key: &T::AccountId,
    ) -> bool;
}
impl<T: Config> EnsureIdtyCallAllowed<T> for () {
    fn create_identity(
        origin: T::Origin,
        _creator: &T::IdtyDid,
        _idty_did: &T::IdtyDid,
        _idty_owner_key: &T::AccountId,
    ) -> bool {
        ensure_root(origin).is_ok()
    }
}

pub trait IdtyData:
    frame_support::Parameter
    + frame_support::pallet_prelude::Member
    + MaybeSerializeDeserialize
    + Debug
    + Default
{
}
impl IdtyData for () {}

pub trait IdtyDid:
    frame_support::Parameter
    + frame_support::pallet_prelude::Member
    + MaybeSerializeDeserialize
    + Debug
    + Default
    + Copy
    + Ord
{
}

pub trait IdtyRight:
    frame_support::Parameter
    + frame_support::pallet_prelude::Member
    + MaybeSerializeDeserialize
    + Debug
    + Default
    + Copy
    + Ord
{
    fn allow_owner_key(self) -> bool;
}

pub trait OnIdtyConfirmed<T: Config> {
    fn on_idty_confirmed(
        idty_did: T::IdtyDid,
        owner_key: T::AccountId,
        removable_on: T::BlockNumber,
    );
}
impl<T: Config> OnIdtyConfirmed<T> for () {
    fn on_idty_confirmed(
        _idty_did: T::IdtyDid,
        _owner_key: T::AccountId,
        _removable_on: T::BlockNumber,
    ) {
    }
}

pub trait OnIdtyValidated<T: Config> {
    fn on_idty_validated(
        idty_did: T::IdtyDid,
        owner_key: T::AccountId,
    ) -> DispatchResultWithPostInfo;
}
impl<T: Config> OnIdtyValidated<T> for () {
    fn on_idty_validated(
        _idty_did: T::IdtyDid,
        _owner_key: T::AccountId,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
}

pub trait OnIdtyRemoved<T: Config> {
    fn on_idty_removed(idty_did: T::IdtyDid, owner_key: T::AccountId) -> Weight;
}
impl<T: Config> OnIdtyRemoved<T> for () {
    fn on_idty_removed(_idty_did: T::IdtyDid, _owner_key: T::AccountId) -> Weight {
        0
    }
}

pub trait OnRightKeyChange<T: Config> {
    fn on_right_key_change(
        idty_did: T::IdtyDid,
        right: T::IdtyRight,
        old_key: Option<T::AccountId>,
        new_key: Option<T::AccountId>,
    );
}

impl<T: Config> OnRightKeyChange<T> for () {
    fn on_right_key_change(
        _idty_did: T::IdtyDid,
        _right: T::IdtyRight,
        _old_key: Option<T::AccountId>,
        _new_key: Option<T::AccountId>,
    ) {
    }
}
