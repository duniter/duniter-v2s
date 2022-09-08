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

//! Defines types and traits for users of pallet membership.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

pub mod traits;

use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub enum Event<IdtyId, MetaData = ()> {
    /// A membership has acquired
    MembershipAcquired(IdtyId, MetaData),
    /// A membership has expired
    MembershipExpired(IdtyId),
    /// A membership has renewed
    MembershipRenewed(IdtyId),
    /// An identity requested membership
    MembershipRequested(IdtyId),
    /// A membership has revoked
    MembershipRevoked(IdtyId),
    /// A pending membership request has expired
    PendingMembershipExpired(IdtyId),
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MembershipData<BlockNumber: Decode + Encode + TypeInfo> {
    pub expire_on: BlockNumber,
}
