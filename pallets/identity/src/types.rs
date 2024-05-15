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

/// internal events related to identity
pub enum IdtyEvent<T: crate::Config> {
    /// IdtyEvent::Created
    /// creation of a new identity by an other
    // pallet account links account to identity
    // pallet wot adds certification
    // pallet quota adds storage item for this identity
    Created {
        creator: T::IdtyIndex,
        owner_key: T::AccountId,
    },
    /// IdtyEvent::Removed
    /// removing an identity (unvalidated or revoked)
    // pallet wot removes associated certifications if status is not revoked
    // pallet quota removes associated quota
    // pallet smith-members exclude smith
    Removed { status: IdtyStatus },
    // TODO add a way to unlink accounts corresponding to revoked or removed identities
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum RevocationReason {
    /// revoked by root (e.g. governance or migration)
    Root,
    /// revoked by user action (revocation document)
    User,
    /// revoked due to inactive period
    Expired,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum RemovalReason {
    /// removed by root
    Root,
    /// removed because unconfirmed
    Unconfirmed,
    /// removed because unvalidated
    Unvalidated,
    /// removed automatically after revocation buffer
    Revoked,
}

/// name of the identity, ascii encoded
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

/// status of the identity
// this is a kind of index to tell the state of the identity
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
    /// created through a first certification but unconfirmed
    #[default]
    Unconfirmed,
    /// confirmed by key owner with a name published but unvalidated
    Unvalidated,
    /// member of the main web of trust
    // (there must be a membership in membership pallet storage)
    Member,
    /// not member of the main web of trust, auto-revocation planned
    NotMember,
    /// revoked manually or automatically, deletion possible
    Revoked,
}

/// identity value (as in key/value)
#[derive(Serialize, Deserialize, Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
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
    /// next action scheduled on identity
    // 0 if no action scheduled
    pub next_scheduled: BlockNumber,
    /// current status of the identity (until validation)
    pub status: IdtyStatus,
}

/// payload to define a new owner key
#[derive(Clone, Copy, Encode, RuntimeDebug)]
pub struct IdtyIndexAccountIdPayload<'a, AccountId, IdtyIndex, Hash> {
    /// hash of the genesis block
    // Avoid replay attack across networks
    pub genesis_hash: &'a Hash,
    /// identity index
    pub idty_index: IdtyIndex,
    /// old owner key of the identity
    pub old_owner_key: &'a AccountId,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub struct RevocationPayload<IdtyIndex, Hash> {
    /// hash of the genesis block
    // Avoid replay attack across networks
    pub genesis_hash: Hash,
    /// identity index
    pub idty_index: IdtyIndex,
}
