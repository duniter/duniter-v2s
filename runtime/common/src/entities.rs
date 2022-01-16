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

pub use pallet_identity::IdtyName;

use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
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
            Self::CreateIdty | Self::LightCert | IdtyRight::StrongCert | Self::Ud => true,
            //IdtyRight::StrongCert => false,
            //_ => false,
        }
    }
    fn create_idty_right() -> Self {
        Self::CreateIdty
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(
    Encode, Decode, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo,
)]
pub struct ValidatorFullIdentification;
