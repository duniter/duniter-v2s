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
use impl_trait_for_tuples::impl_for_tuples;

pub trait CheckIdtyCallAllowed<T: Config> {
    fn check_create_identity(creator: T::IdtyIndex) -> Result<(), DispatchError>;
}

#[impl_for_tuples(5)]
impl<T: Config> CheckIdtyCallAllowed<T> for Tuple {
    fn check_create_identity(creator: T::IdtyIndex) -> Result<(), DispatchError> {
        for_tuples!( #( Tuple::check_create_identity(creator)?; )* );
        Ok(())
    }
}

pub trait IdtyNameValidator {
    fn validate(idty_name: &IdtyName) -> bool;
}

pub trait OnIdtyChange<T: Config> {
    fn on_idty_change(idty_index: T::IdtyIndex, idty_event: &IdtyEvent<T>);
}

#[impl_for_tuples(5)]
#[allow(clippy::let_and_return)]
impl<T: Config> OnIdtyChange<T> for Tuple {
    fn on_idty_change(idty_index: T::IdtyIndex, idty_event: &IdtyEvent<T>) {
        for_tuples!( #( Tuple::on_idty_change(idty_index, idty_event); )* );
    }
}

/// trait used to link an account to an identity
pub trait LinkIdty<AccountId, IdtyIndex> {
    fn link_identity(account_id: &AccountId, idty_index: IdtyIndex) -> Result<(), DispatchError>;
}
impl<AccountId, IdtyIndex> LinkIdty<AccountId, IdtyIndex> for () {
    fn link_identity(_: &AccountId, _: IdtyIndex) -> Result<(), DispatchError> {
        Ok(())
    }
}
