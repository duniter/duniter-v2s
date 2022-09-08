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
use sp_runtime::traits::Saturating;

pub trait EnsureIdtyCallAllowed<T: Config> {
    fn can_create_identity(creator: T::IdtyIndex) -> bool;
    fn can_confirm_identity(idty_index: T::IdtyIndex) -> bool;
    fn can_validate_identity(idty_index: T::IdtyIndex) -> bool;
    fn can_change_identity_address(idty_index: T::IdtyIndex) -> bool;
    fn can_remove_identity(idty_index: T::IdtyIndex) -> bool;
}

#[impl_for_tuples(5)]
#[allow(clippy::let_and_return)]
impl<T: Config> EnsureIdtyCallAllowed<T> for Tuple {
    fn can_create_identity(creator: T::IdtyIndex) -> bool {
        for_tuples!( #( if !Tuple::can_create_identity(creator) { return false; } )* );
        true
    }
    fn can_confirm_identity(idty_index: T::IdtyIndex) -> bool {
        for_tuples!( #( if !Tuple::can_confirm_identity(idty_index) { return false; } )* );
        true
    }
    fn can_validate_identity(idty_index: T::IdtyIndex) -> bool {
        for_tuples!( #( if !Tuple::can_validate_identity(idty_index) { return false; } )* );
        true
    }
    fn can_change_identity_address(idty_index: T::IdtyIndex) -> bool {
        for_tuples!( #( if !Tuple::can_change_identity_address(idty_index) { return false; } )* );
        true
    }
    fn can_remove_identity(idty_index: T::IdtyIndex) -> bool {
        for_tuples!( #( if !Tuple::can_remove_identity(idty_index) { return false; } )* );
        true
    }
}

pub trait IdtyNameValidator {
    fn validate(idty_name: &IdtyName) -> bool;
}

pub trait OnIdtyChange<T: Config> {
    fn on_idty_change(idty_index: T::IdtyIndex, idty_event: &IdtyEvent<T>) -> Weight;
}

#[impl_for_tuples(5)]
#[allow(clippy::let_and_return)]
impl<T: Config> OnIdtyChange<T> for Tuple {
    fn on_idty_change(idty_index: T::IdtyIndex, idty_event: &IdtyEvent<T>) -> Weight {
        let mut weight = 0;
        for_tuples!( #( weight = weight.saturating_add(Tuple::on_idty_change(idty_index, idty_event)); )* );
        weight
    }
}

pub trait RemoveIdentityConsumers<IndtyIndex> {
    fn remove_idty_consumers(idty_index: IndtyIndex) -> Weight;
}
impl<IndtyIndex> RemoveIdentityConsumers<IndtyIndex> for () {
    fn remove_idty_consumers(_: IndtyIndex) -> Weight {
        0
    }
}
