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

//! Various basic types for use in the certification pallet.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_staking::SessionIndex;

#[cfg_attr(feature = "std", derive(Debug, Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct MemberData<AccountId> {
    /// session at which the membership expires
    pub expire_on_session: SessionIndex,
    /// session before which the member must have rotated keys
    pub must_rotate_keys_before: SessionIndex,
    /// pubkey of the member
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
