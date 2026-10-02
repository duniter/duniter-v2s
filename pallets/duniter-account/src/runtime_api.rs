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

sp_api::decl_runtime_apis! {
    /// Runtime API for duniter account pallet
    #[api_version(2)]
    pub trait DuniterAccountApi<Balance>
    where
        EstimatedCost<Balance>: Codec,
        EstimatedCostV1<Balance>: Codec,
    {
        /// Simulate the maximum cost of an extrinsic
        ///
        /// Returns the version 1 estimate without the available quota.
        #[changed_in(2)]
        fn estimate_cost(
            uxt: Block::Extrinsic,
        ) -> EstimatedCostV1<Balance>;

        /// Estimate the pre-dispatch cost of an extrinsic.
        ///
        /// Returns the estimated refundable fees, the refund covered by the
        /// signer's quota, the remaining cost, and the available quota.
        /// The estimate excludes the tip and does not validate the signature,
        /// nonce, account balance, or other transaction validity rules.
        fn estimate_cost(
            uxt: Block::Extrinsic,
        ) -> EstimatedCost<Balance>;
    }
}

/// Estimated transaction cost returned by version 1 of the runtime API.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Debug)]
pub struct EstimatedCostV1<Balance> {
    /// The estimated effective cost for the user (fees - refund).
    pub cost: Balance,
    /// The estimated refundable fees for the extrinsic, excluding its tip.
    pub fees: Balance,
    /// The quota available to refund transaction fees.
    pub refund: Balance,
}

/// Estimated transaction cost.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Debug)]
pub struct EstimatedCost<Balance> {
    /// The estimated effective cost for the user (fees - refund), excluding the tip.
    pub cost: Balance,
    /// The estimated refundable fees for the extrinsic, excluding its tip.
    pub fees: Balance,
    /// The estimated refund, limited by refundable fees and available quota.
    pub refund: Balance,
    /// The quota available to refund transaction fees.
    pub available_quota: Balance,
}
