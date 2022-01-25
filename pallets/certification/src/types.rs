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

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct CertValue<BlockNumber> {
    pub renewable_on: BlockNumber,
    pub removable_on: BlockNumber,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct IdtyCertMeta<BlockNumber: Default> {
    pub issued_count: u32,
    pub next_issuable_on: BlockNumber,
    pub received_count: u32,
}

impl<BlockNumber: Default> Default for IdtyCertMeta<BlockNumber> {
    fn default() -> Self {
        Self {
            issued_count: 0,
            next_issuable_on: BlockNumber::default(),
            received_count: 0,
        }
    }
}
