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

use crate::*;

/// Trait for managing refund operations.
pub trait RefundFee<T: Config> {
    /// Request a refund of a fee for a specific account and identity.
    fn request_refund(account: T::AccountId, identity: IdtyId<T>, amount: BalanceOf<T>);
}

impl<T: Config> RefundFee<T> for () {
    fn request_refund(_account: T::AccountId, _identity: IdtyId<T>, _amount: BalanceOf<T>) {}
}
