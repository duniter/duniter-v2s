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

use super::{AccountId, Identity, IdtyIndex, IdtyRight, Origin, Runtime, UdAccountsStorage};
use frame_support::pallet_prelude::DispatchResultWithPostInfo;

pub struct OnIdtyValidatedHandler;
impl pallet_identity::traits::OnIdtyValidated<Runtime> for OnIdtyValidatedHandler {
    fn on_idty_validated(
        idty_index: IdtyIndex,
        _owner_key: AccountId,
    ) -> DispatchResultWithPostInfo {
        Identity::add_right(Origin::root(), idty_index, IdtyRight::Ud)
    }
}

pub struct OnRightKeyChangeHandler;
impl pallet_identity::traits::OnRightKeyChange<Runtime> for OnRightKeyChangeHandler {
    fn on_right_key_change(
        _idty_index: IdtyIndex,
        right: IdtyRight,
        old_key_opt: Option<AccountId>,
        new_key_opt: Option<AccountId>,
    ) {
        match right {
            IdtyRight::Ud => UdAccountsStorage::replace_account(old_key_opt, new_key_opt),
            IdtyRight::CreateIdty => 0,
            IdtyRight::LightCert => 0,
            IdtyRight::StrongCert => 0,
        };
    }
}
