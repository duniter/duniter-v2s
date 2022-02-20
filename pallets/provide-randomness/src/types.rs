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

//! Various basic types for use in pallet provide randomness

use super::RequestId;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::H256;

#[derive(Clone, Copy, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RandomnessType {
    RandomnessFromPreviousBlock,
    RandomnessFromOneEpochAgo,
    RandomnessFromTwoEpochsAgo,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Request {
    pub request_id: RequestId,
    pub salt: H256,
}
