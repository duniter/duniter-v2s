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

use pallet_duniter_wot::IdtyRight;

pub struct OnRightKeyChangeHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
        IdtyIndex,
        Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex, IdtyRight = IdtyRight>
            + pallet_ud_accounts_storage::Config,
    > pallet_identity::traits::OnRightKeyChange<Runtime> for OnRightKeyChangeHandler<Runtime>
{
    fn on_right_key_change(
        _idty_index: IdtyIndex,
        right: Runtime::IdtyRight,
        old_key_opt: Option<Runtime::AccountId>,
        new_key_opt: Option<Runtime::AccountId>,
    ) {
        match right {
            IdtyRight::Ud => <pallet_ud_accounts_storage::Pallet<Runtime>>::replace_account(
                old_key_opt,
                new_key_opt,
            ),
            IdtyRight::CreateIdty => 0,
            IdtyRight::LightCert => 0,
            IdtyRight::StrongCert => 0,
        };
    }
}
