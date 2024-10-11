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
use frame_support::pallet_prelude::{RuntimeDebug, Weight};

use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// Represent membership-related events.
pub enum Event<IdtyId> {
    /// A membership was acquired.
    MembershipAdded(IdtyId),
    /// A membership was terminated.
    MembershipRemoved(IdtyId),
    /// A membership was renewed.
    MembershipRenewed(IdtyId),
}

/// Represent membership data.
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
pub struct MembershipData<BlockNumber: Decode + Encode + TypeInfo> {
    pub expire_on: BlockNumber,
}

impl<IdtyId> traits::OnNewMembership<IdtyId> for () {
    fn on_created(_idty_index: &IdtyId) {}

    fn on_renewed(_idty_index: &IdtyId) {}
}

impl<IdtyId> traits::OnRemoveMembership<IdtyId> for () {
    fn on_removed(_idty_index: &IdtyId) -> Weight {
        Weight::zero()
    }
}
