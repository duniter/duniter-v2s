// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub enum IdtyRight {
    CreateIdty,
    LightCert,
    StrongCert,
    Ud,
}
impl Default for IdtyRight {
    fn default() -> Self {
        Self::Ud
    }
}
impl pallet_identity::traits::IdtyRight for IdtyRight {
    fn allow_owner_key(self) -> bool {
        match self {
            Self::CreateIdty | Self::LightCert | Self::Ud => true,
            IdtyRight::StrongCert => false,
            //_ => false,
        }
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub struct IdtyDid {
    pub hash: H256,
    pub planet: Planet,
    pub latitude: u32,
    pub longitude: u32,
}
impl PartialOrd for IdtyDid {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.hash.partial_cmp(&other.hash) {
            Some(core::cmp::Ordering::Equal) => match self.planet.partial_cmp(&other.planet) {
                Some(core::cmp::Ordering::Equal) => {
                    match self.latitude.partial_cmp(&other.latitude) {
                        Some(core::cmp::Ordering::Equal) => {
                            self.longitude.partial_cmp(&other.longitude)
                        }
                        o => o,
                    }
                }
                o => o,
            },
            o => o,
        }
    }
}
impl Ord for IdtyDid {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.hash.cmp(&other.hash) {
            core::cmp::Ordering::Equal => match self.planet.cmp(&other.planet) {
                core::cmp::Ordering::Equal => match self.latitude.cmp(&other.latitude) {
                    core::cmp::Ordering::Equal => self.longitude.cmp(&other.longitude),
                    o => o,
                },
                o => o,
            },
            o => o,
        }
    }
}
impl pallet_identity::traits::IdtyDid for IdtyDid {}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub enum Planet {
    Earth,
}
impl Default for Planet {
    fn default() -> Self {
        Self::Earth
    }
}
