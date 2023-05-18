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

//! Various basic types for use in the identity pallet.

use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

/// events related to identity
pub enum IdtyEvent<T: crate::Config> {
    /// creation of a new identity by an other
    Created { creator: T::IdtyIndex },
    /// confirmation of an identity (with a given name)
    Confirmed,
    /// validation of an identity
    Validated,
    /// changing the owner key of the identity
    ChangedOwnerKey { new_owner_key: T::AccountId },
    /// removing an identity
    Removed { status: IdtyStatus },
}

/// name of the identity, ascii encoded
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub struct IdtyName(pub Vec<u8>);

/// implement scale string typeinfo for encoding
impl scale_info::TypeInfo for IdtyName {
    type Identity = str;

    fn type_info() -> scale_info::Type {
        Self::Identity::type_info()
    }
}

#[cfg(feature = "std")]
impl From<&str> for IdtyName {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for IdtyName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        std::str::from_utf8(&self.0)
            .map_err(|e| serde::ser::Error::custom(format!("{:?}", e)))?
            .serialize(serializer)
    }
}

#[cfg(feature = "std")]
impl<'de> serde::Deserialize<'de> for IdtyName {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self(String::deserialize(de)?.as_bytes().to_vec()))
    }
}

/// status of the identity
/// used for temporary period before validation
/// also used for buffer when losing membership before being deleted
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum IdtyStatus {
    /// created through a first certification
    Created,
    /// confirmed by owner with a name published
    ConfirmedByOwner,
    /// validated by the main web of trust
    Validated,
    // disabled by the main web of trust, deletion planned
    // Disabled,
}
impl Default for IdtyStatus {
    fn default() -> Self {
        IdtyStatus::Created
    }
}

/// identity value (as in key/value)
#[cfg_attr(feature = "std", derive(Debug, Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct IdtyValue<BlockNumber, AccountId, IdtyData> {
    /// data shared between pallets defined by runtime
    /// only contains first_eligible_ud in our case
    pub data: IdtyData,
    /// block before which creating a new identity is not allowed
    pub next_creatable_identity_on: BlockNumber,
    /// previous owner key of this identity (optional)
    pub old_owner_key: Option<(AccountId, BlockNumber)>,
    /// current owner key of this identity
    pub owner_key: AccountId,
    /// block before which this identity can not be removed
    /// used only for temporary period before validation
    /// equals 0 for a validated identity
    pub removable_on: BlockNumber,
    /// current status of the identity (until validation)
    pub status: IdtyStatus,
}

/// payload to define a new owner key
#[derive(Clone, Copy, Encode, RuntimeDebug)]
pub struct NewOwnerKeyPayload<'a, AccountId, IdtyIndex, Hash> {
    /// hash of the genesis block
    // Avoid replay attack between networks
    pub genesis_hash: &'a Hash,
    /// identity index
    pub idty_index: IdtyIndex,
    /// old owner key of the identity
    pub old_owner_key: &'a AccountId,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub struct RevocationPayload<IdtyIndex, Hash> {
    /// hash of the genesis block
    // Avoid replay attack between networks
    pub genesis_hash: Hash,
    /// identity index
    pub idty_index: IdtyIndex,
}
