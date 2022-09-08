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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::Zero;

#[derive(Clone, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AccountData<Balance> {
    pub(super) random_id: Option<H256>,
    pub(super) free: Balance,
    pub(super) reserved: Balance,
    fee_frozen: Balance,
}

impl<Balance: Zero> AccountData<Balance> {
    pub fn set_balances(&mut self, new_balances: pallet_balances::AccountData<Balance>) {
        self.free = new_balances.free;
        self.reserved = new_balances.reserved;
        self.fee_frozen = new_balances.fee_frozen;
    }
    pub fn was_providing(&self) -> bool {
        !self.free.is_zero() || !self.reserved.is_zero()
    }
}

impl<Balance: Zero> From<AccountData<Balance>> for pallet_balances::AccountData<Balance> {
    fn from(account_data: AccountData<Balance>) -> Self {
        Self {
            free: account_data.free,
            reserved: account_data.reserved,
            misc_frozen: Zero::zero(),
            fee_frozen: account_data.fee_frozen,
        }
    }
}

#[derive(Clone, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct GenesisAccountData<Balance> {
    pub random_id: H256,
    pub balance: Balance,
    pub is_identity: bool,
}
