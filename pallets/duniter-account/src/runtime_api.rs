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

use codec::{Codec, Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

sp_api::decl_runtime_apis! {
    /// Runtime API for duniter account pallet
    pub trait DuniterAccountApi<Balance>
    where
        EstimatedCost<Balance>: Codec,
    {
        /// Simulate the maximum cost of an extrinsic
        ///
        /// Returns an object with two fields:
        /// - `max_cost`: estimated effective cost for the user (fees - refund)
        /// - `max_fees`: estimated amount of fees for the extrinsic
        /// - `min_refund`: estimated amount of refund from quota
        fn estimate_cost(
            uxt: Block::Extrinsic,
        ) -> EstimatedCost<Balance>;
    }
}

/// Account total balance information
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, RuntimeDebug)]
pub struct EstimatedCost<Balance> {
    /// The estimated effective cost for the user (fees - refund)
    pub cost: Balance,
    /// The estimated amount of fees for the extrinsic
    pub fees: Balance,
    /// The estimated amount of refund from quota
    pub refund: Balance,
}
