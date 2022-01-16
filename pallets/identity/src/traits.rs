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

use crate::*;
use frame_support::pallet_prelude::*;
use sp_runtime::traits::MaybeSerializeDeserialize;
use sp_std::fmt::Debug;

pub trait EnsureIdtyCallAllowed<T: Config> {
    fn can_create_identity(creator: T::IdtyIndex) -> bool;
}

impl<T: Config> EnsureIdtyCallAllowed<T> for () {
    fn can_create_identity(_creator: T::IdtyIndex) -> bool {
        true
    }
}

pub trait ProvideIdtyData<T: Config> {
    fn provide_identity_data(
        creator: T::IdtyIndex,
        idty_name: &IdtyName,
        idty_owner_key: &T::AccountId,
    ) -> T::IdtyData;
}

impl<T: Config> ProvideIdtyData<T> for ()
where
    T::IdtyData: Default,
{
    fn provide_identity_data(
        _creator: T::IdtyIndex,
        _idty_name: &IdtyName,
        _idty_owner_key: &T::AccountId,
    ) -> T::IdtyData {
        Default::default()
    }
}

pub trait IdtyNameValidator {
    fn validate(idty_name: &IdtyName) -> bool;
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
    fn create_idty_right() -> Self;
}

pub enum IdtyEvent<T: Config> {
    Created { creator: T::IdtyIndex },
    Confirmed,
    Validated,
    Expired,
    Removed,
}

pub trait OnIdtyChange<T: Config> {
    fn on_idty_change(idty_index: T::IdtyIndex, idty_event: IdtyEvent<T>) -> Weight;
}
impl<T: Config> OnIdtyChange<T> for () {
    fn on_idty_change(_idty_index: T::IdtyIndex, _idty_event: IdtyEvent<T>) -> Weight {
        0
    }
}

pub trait OnRightKeyChange<T: Config> {
    fn on_right_key_change(
        idty_index: T::IdtyIndex,
        right: T::IdtyRight,
        old_key: Option<T::AccountId>,
        new_key: Option<T::AccountId>,
    );
}

impl<T: Config> OnRightKeyChange<T> for () {
    fn on_right_key_change(
        _idty_index: T::IdtyIndex,
        _right: T::IdtyRight,
        _old_key: Option<T::AccountId>,
        _new_key: Option<T::AccountId>,
    ) {
    }
}
