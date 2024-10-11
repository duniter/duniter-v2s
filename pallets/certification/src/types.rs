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
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

/// Represents the certification metadata attached to an identity.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct IdtyCertMeta<BlockNumber: Default> {
    /// Number of certifications issued by this identity.
    pub issued_count: u32,
    /// Block number before which the identity is not allowed to issue a new certification.
    pub next_issuable_on: BlockNumber,
    /// Number of certifications received by this identity.
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
