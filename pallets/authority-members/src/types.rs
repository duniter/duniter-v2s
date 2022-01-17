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
pub struct MemberData<ValidatorId: Decode + Encode + TypeInfo> {
    pub expire_on_session: SessionIndex,
    pub session_keys_provided: bool,
    pub validator_id: ValidatorId,
}

impl<ValidatorId: Decode + Encode + TypeInfo> MemberData<ValidatorId> {
    pub fn new(validator_id: ValidatorId, expire_on_session: SessionIndex) -> Self {
        MemberData {
            expire_on_session,
            session_keys_provided: false,
            validator_id,
        }
    }
    pub fn new_genesis(validator_id: ValidatorId) -> Self {
        MemberData {
            expire_on_session: 0,
            session_keys_provided: true,
            validator_id,
        }
    }
}
