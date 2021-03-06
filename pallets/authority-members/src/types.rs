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

//! Various basic types for use in the certification pallet.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_staking::SessionIndex;

#[cfg_attr(feature = "std", derive(Debug, Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct MemberData<AccountId> {
    pub expire_on_session: SessionIndex,
    pub must_rotate_keys_before: SessionIndex,
    pub owner_key: AccountId,
}

impl<AccountId> MemberData<AccountId> {
    pub fn new_genesis(must_rotate_keys_before: SessionIndex, owner_key: AccountId) -> Self {
        MemberData {
            expire_on_session: 0,
            must_rotate_keys_before,
            owner_key,
        }
    }
}
