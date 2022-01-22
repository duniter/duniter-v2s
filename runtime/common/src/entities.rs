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

			#[derive(Clone, codec::Decode, Debug, codec::Encode, Eq, PartialEq)]
			pub struct SessionKeysWrapper(pub SessionKeys);

			impl From<SessionKeysWrapper> for SessionKeys {
				fn from(keys_wrapper: SessionKeysWrapper) -> SessionKeys {
					keys_wrapper.0
				}
			}

			impl scale_info::TypeInfo for SessionKeysWrapper {
				type Identity = [u8; 128];

				fn type_info() -> scale_info::Type {
					Self::Identity::type_info()
				}
			}
		}
	}
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SmithsMembershipMetaData<SessionKeysWrapper> {
    pub p2p_endpoint: sp_runtime::RuntimeString,
    pub session_keys: SessionKeysWrapper,
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(
    Encode, Decode, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo,
)]
pub struct ValidatorFullIdentification;
