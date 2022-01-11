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
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(
    Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo,
)]
pub struct IdtyName(pub sp_std::vec::Vec<u8>);

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum IdtyStatus {
    Created,
    ConfirmedByOwner,
    Validated,
    Expired,
}
impl Default for IdtyStatus {
    fn default() -> Self {
        IdtyStatus::Created
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct IdtyValue<
    AccountId: Decode + Encode + TypeInfo,
    BlockNumber: Decode + Encode + TypeInfo,
    IdtyData: Decode + Encode + TypeInfo,
    IdtyRight: Decode + Encode + TypeInfo,
> {
    pub name: IdtyName,
    pub expire_on: BlockNumber,
    pub owner_key: AccountId,
    pub removable_on: BlockNumber,
    pub renewable_on: BlockNumber,
    pub rights: Vec<(IdtyRight, Option<AccountId>)>,
    pub status: IdtyStatus,
    pub data: IdtyData,
}

impl<AccountId, BlockNumber, IdtyData, IdtyRight>
    IdtyValue<AccountId, BlockNumber, IdtyData, IdtyRight>
where
    AccountId: Clone + Decode + Encode + TypeInfo,
    BlockNumber: Decode + Encode + TypeInfo,
    IdtyData: Decode + Encode + TypeInfo,
    IdtyRight: crate::traits::IdtyRight + Decode + Encode + TypeInfo,
{
    pub fn get_right_key(&self, right: IdtyRight) -> Option<AccountId> {
        if let Ok(index) = self
            .rights
            .binary_search_by(|(right_, _)| right_.cmp(&right))
        {
            if self.rights[index].1.is_some() {
                self.rights[index].1.clone()
            } else if right.allow_owner_key() {
                Some(self.owner_key.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}
