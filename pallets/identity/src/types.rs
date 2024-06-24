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
use serde::{Deserialize, Serialize};

/// Internal events related to identity.
pub enum IdtyEvent<T: crate::Config> {
    /// Creation of a new identity by another.
    // pallet account links account to identity
    // pallet wot adds certification
    // pallet quota adds storage item for this identity
    Created {
        /// Identity of the creator.
        creator: T::IdtyIndex,
        /// Account of the identity owner.
        owner_key: T::AccountId,
    },
    /// Removing an identity (unvalidated or revoked).
    // pallet wot removes associated certifications if status is not revoked
    // pallet quota removes associated quota
    // pallet smith-members exclude smith
    Removed {
        /// Status of the identity.
        status: IdtyStatus,
    },
    // TODO add a way to unlink accounts corresponding to revoked or removed identities
}

/// Reasons for revocation.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum RevocationReason {
    /// Revoked by root (e.g., governance or migration).
    Root,
    /// Revoked by user action (revocation document).
    User,
    /// Revoked due to inactive period.
    Expired,
}

/// Reasons for removal.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum RemovalReason {
    /// Removed by root.
    Root,
    /// Removed because unconfirmed.
    Unconfirmed,
    /// Removed because unvalidated.
    Unvalidated,
    /// Removed automatically after revocation buffer.
    Revoked,
}

/// Represents the name of an identity, ASCII encoded.
#[derive(
    Encode,
    Decode,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    RuntimeDebug,
    Serialize,
    Deserialize,
    TypeInfo,
)]
pub struct IdtyName(pub sp_std::vec::Vec<u8>);

impl From<&str> for IdtyName {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

/// State of an identity.
#[derive(
    Encode,
    Decode,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    Deserialize,
    Serialize,
)]
pub enum IdtyStatus {
    /// Created through a first certification but unconfirmed.
    #[default]
    Unconfirmed,
    /// Confirmed by key owner with a name published but unvalidated.
    Unvalidated,
    /// Member of the main web of trust.
    // (there must be a membership in membership pallet storage)
    Member,
    /// Not a member of the main web of trust, auto-revocation planned.
    NotMember,
    /// Revoked manually or automatically, deletion possible.
    Revoked,
}

/// Identity value structure.
///
/// Represents the value associated with an identity, akin to key/value pairs.
#[derive(Serialize, Deserialize, Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct IdtyValue<BlockNumber, AccountId, IdtyData> {
    /// Data shared between pallets defined by runtime.
    /// Only contains `first_eligible_ud` in our case.
    pub data: IdtyData,
    /// Block before which creating a new identity is not allowed.
    pub next_creatable_identity_on: BlockNumber,
    /// Previous owner key of this identity (optional).
    pub old_owner_key: Option<(AccountId, BlockNumber)>,
    /// Current owner key of this identity.
    pub owner_key: AccountId,
    /// Next action scheduled on identity.
    ///
    /// `0` if no action is scheduled.
    pub next_scheduled: BlockNumber,
    /// Current status of the identity (until validation).
    pub status: IdtyStatus,
}

/// Reprensent the payload to define a new owner key.
#[derive(Clone, Copy, Encode, RuntimeDebug)]
pub struct IdtyIndexAccountIdPayload<'a, AccountId, IdtyIndex, Hash> {
    /// Hash of the genesis block.
    // Used to avoid replay attacks across networks.
    pub genesis_hash: &'a Hash,
    /// Identity index.
    pub idty_index: IdtyIndex,
    /// Old owner key of the identity.
    pub old_owner_key: &'a AccountId,
}

/// Represents the payload for identity revocation.
#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub struct RevocationPayload<IdtyIndex, Hash> {
    /// Hash of the genesis block.
    // Used to avoid replay attacks across networks.
    pub genesis_hash: Hash,
    /// Identity index.
    pub idty_index: IdtyIndex,
}
