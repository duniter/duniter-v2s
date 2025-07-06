// Copyright 2023 Axiom-Team
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

//! Defines types and traits for users of pallet distance.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::pallet_prelude::RuntimeDebug;
use scale_info::TypeInfo;
use sp_inherents::{InherentIdentifier, IsFatalError};
use sp_runtime::Perbill;
#[cfg(feature = "std")]
use std::marker::PhantomData;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"distanc0";

/// Represents the result of a distance computation.
#[derive(Clone, DecodeWithMemTracking, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ComputationResult {
    pub distances: scale_info::prelude::vec::Vec<Perbill>,
}

/// Errors that can occur while checking the inherent data in `ProvideInherent::check_inherent` from pallet-distance.
#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode, thiserror::Error))]
pub enum InherentError {}

impl IsFatalError for InherentError {
    fn is_fatal_error(&self) -> bool {
        false
    }
}

#[cfg(feature = "std")]
pub struct InherentDataProvider<IdtyIndex: Decode + Encode + PartialEq + TypeInfo> {
    computation_result: Option<ComputationResult>,
    _p: PhantomData<IdtyIndex>,
}

#[cfg(feature = "std")]
impl<IdtyIndex: Decode + Encode + PartialEq + TypeInfo> InherentDataProvider<IdtyIndex> {
    pub fn new(computation_result: Option<ComputationResult>) -> Self {
        Self {
            computation_result,
            _p: PhantomData,
        }
    }
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl<IdtyIndex: Decode + Encode + PartialEq + TypeInfo + Send + Sync>
    sp_inherents::InherentDataProvider for InherentDataProvider<IdtyIndex>
{
    async fn provide_inherent_data(
        &self,
        inherent_data: &mut sp_inherents::InherentData,
    ) -> Result<(), sp_inherents::Error> {
        if let Some(computation_result) = &self.computation_result {
            inherent_data.put_data(INHERENT_IDENTIFIER, computation_result)?;
        }
        Ok(())
    }

    async fn try_handle_error(
        &self,
        _identifier: &InherentIdentifier,
        _error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        // No errors occur here.
        // Errors handled here are emitted in the `ProvideInherent::check_inherent`
        // (from pallet-distance) which is not implemented.
        None
    }
}
