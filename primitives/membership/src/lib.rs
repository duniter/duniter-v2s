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
use frame_support::pallet_prelude::RuntimeDebug;

use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// membership events
pub enum Event<IdtyId> {
    /// A membership was acquired.
    MembershipAdded(IdtyId),
    /// A membership was terminated.
    MembershipRemoved(IdtyId),
    /// A membership was renewed.
    MembershipRenewed(IdtyId),
}

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

use impl_trait_for_tuples::impl_for_tuples;
// use sp_std::prelude::*;
// use frame_support::pallet_prelude::*;
// use frame_system::pallet_prelude::*;

#[impl_for_tuples(5)]
impl<IdtyId> traits::OnEvent<IdtyId> for Tuple {
    fn on_event(event: &Event<IdtyId>) {
        for_tuples!( #( Tuple::on_event(event); )* );
    }
}
