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

// see `struct AccountData` for details in substrate code
#[derive(Clone, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AccountData<Balance> {
    /// A random identifier that can not be chosen by the user
    // this intends to be used as a robust identification system
    pub(super) random_id: Option<H256>,
    // see Substrate AccountData
    pub(super) free: Balance,
    // see Substrate AccountData
    pub(super) reserved: Balance,
    // see Substrate AccountData
    fee_frozen: Balance,
}

impl<Balance: Zero> AccountData<Balance> {
    pub fn set_balances(&mut self, new_balances: pallet_balances::AccountData<Balance>) {
        self.free = new_balances.free;
        self.reserved = new_balances.reserved;
        self.fee_frozen = new_balances.frozen;
    }
}

// convert Duniter AccountData to Balances AccountData
// needed for trait implementation
impl<Balance: Zero> From<AccountData<Balance>> for pallet_balances::AccountData<Balance> {
    fn from(account_data: AccountData<Balance>) -> Self {
        Self {
            free: account_data.free,
            reserved: account_data.reserved,
            frozen: account_data.fee_frozen,
            flags: Default::default(), // default flags since not used
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
