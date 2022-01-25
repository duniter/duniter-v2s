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

use crate::IdtyIndex;
use frame_support::pallet_prelude::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MembershipMetaData<AccountId>(pub AccountId);
impl<AccountId: Eq> sp_membership::traits::Validate<AccountId> for MembershipMetaData<AccountId> {
    fn validate(&self, account_id: &AccountId) -> bool {
        &self.0 == account_id
    }
}
/*impl From<AccountId> for MembershipMetaData {
    fn from(account_id: AccountId) -> Self {
        Self(account_id)
    }
}*/
/*impl Into<AccountId> for MembershipMetaData {
    fn into(self) -> AccountId {
        self.0
    }
}*/

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(codec::Decode, codec::Encode, Eq, PartialEq, TypeInfo)]
pub enum WotDiff {
    AddNode(IdtyIndex),
    AddPendingLink(IdtyIndex, IdtyIndex),
    AddLink(IdtyIndex, IdtyIndex),
    DelLink(IdtyIndex, IdtyIndex),
    DisableNode(IdtyIndex),
}

impl Default for WotDiff {
    fn default() -> Self {
        unreachable!()
    }
}
