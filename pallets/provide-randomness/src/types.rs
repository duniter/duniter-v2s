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

//! Various basic types for use in pallet provide randomness

use super::RequestId;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::H256;

/// The type of randomness source.
#[derive(Clone, Copy, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RandomnessType {
    /// Randomness derived from the previous block.
    RandomnessFromPreviousBlock,
    /// Randomness derived from one epoch ago.
    RandomnessFromOneEpochAgo,
    /// Randomness derived from two epochs ago.
    RandomnessFromTwoEpochsAgo,
}

/// Represents a randomness request.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Request {
    /// Request ID.
    pub request_id: RequestId,
    /// Salt used for the request.
    pub salt: H256,
}
