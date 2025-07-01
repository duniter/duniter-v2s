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
    /// Runtime API for Universal Dividend pallet
    pub trait UniversalDividendApi<AccountId, Balance>
    where
        AccountId: Codec,
        AccountBalances<Balance>: Codec,
    {
        /// Get the total balance information for an account
        ///
        /// Returns an object with three fields:
        /// - `total`: total balance
        /// - `transferable`: sum of reducible + unclaim_uds
        /// - `unclaim_uds`: amount of unclaimed UDs
        fn account_balances(account: AccountId) -> AccountBalances<Balance>;
    }
}

/// Account total balance information
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, RuntimeDebug)]
pub struct AccountBalances<Balance> {
    /// The total amount of funds for which the user is the ultimate beneficial owner.
    /// Includes funds that may not be transferable (e.g., reserved balance, existential deposit).
    pub total: Balance,
    /// The maximum amount of funds that can be successfully withdrawn or transferred
    /// (includes unclaimed UDs).
    pub transferable: Balance,
    /// The total amount of unclaimed UDs (accounts for any re-evaluations of UDs).
    pub unclaim_uds: Balance,
}
