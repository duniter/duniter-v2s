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

use codec::{Decode, Encode};
use frame_support::pallet_prelude::RuntimeDebug;
use scale_info::TypeInfo;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError};
use sp_runtime::Perbill;
#[cfg(feature = "std")]
use std::marker::PhantomData;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"distanc0";

/// Represents the result of a distance computation.
#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ComputationResult {
    pub distances: scale_info::prelude::vec::Vec<Perbill>,
}

#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode, thiserror::Error))]
pub enum InherentError {
    #[cfg_attr(feature = "std", error("InvalidComputationResultFile"))]
    InvalidComputationResultFile,
}

impl IsFatalError for InherentError {
    fn is_fatal_error(&self) -> bool {
        match self {
            InherentError::InvalidComputationResultFile => false,
        }
    }
}

impl InherentError {
    #[cfg(feature = "std")]
    pub fn try_from(id: &InherentIdentifier, mut data: &[u8]) -> Option<Self> {
        if id == &INHERENT_IDENTIFIER {
            <InherentError as codec::Decode>::decode(&mut data).ok()
        } else {
            None
        }
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
        inherent_data: &mut InherentData,
    ) -> Result<(), sp_inherents::Error> {
        if let Some(computation_result) = &self.computation_result {
            inherent_data.put_data(INHERENT_IDENTIFIER, computation_result)?;
        }
        Ok(())
    }

    async fn try_handle_error(
        &self,
        identifier: &InherentIdentifier,
        error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        if *identifier != INHERENT_IDENTIFIER {
            return None;
        }

        Some(Err(sp_inherents::Error::Application(Box::from(
            InherentError::try_from(&INHERENT_IDENTIFIER, error)?,
        ))))
    }
}
