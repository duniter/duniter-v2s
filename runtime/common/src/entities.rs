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

use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

#[macro_export]
macro_rules! declare_session_keys {
    {} => {
        pub mod opaque {
            use super::*;

            impl_opaque_keys! {
                pub struct SessionKeys {
                    pub grandpa: Grandpa,
                    pub babe: Babe,
                    pub im_online: ImOnline,
                    pub authority_discovery: AuthorityDiscovery,
                }
            }
        }
    }
}

#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Default,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct IdtyData {
    /// number of the first claimable UD
    pub first_eligible_ud: pallet_universal_dividend::FirstEligibleUd,
}

impl From<IdtyData> for pallet_universal_dividend::FirstEligibleUd {
    fn from(idty_data: IdtyData) -> Self {
        idty_data.first_eligible_ud
    }
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    RuntimeDebug,
    TypeInfo,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ValidatorFullIdentification;
