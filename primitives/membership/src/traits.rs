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

use frame_support::pallet_prelude::*;

/// A trait defining operations for checking if membership-related operations are allowed.
pub trait CheckMembershipOpAllowed<IdtyId> {
    /// Checks if adding a membership is allowed.
    fn check_add_membership(idty_id: IdtyId) -> Result<(), DispatchError>;
    /// Checks if renewing a membership is allowed.
    fn check_renew_membership(idty_id: IdtyId) -> Result<(), DispatchError>;
}

impl<IdtyId> CheckMembershipOpAllowed<IdtyId> for () {
    fn check_add_membership(_: IdtyId) -> Result<(), DispatchError> {
        Ok(())
    }

    fn check_renew_membership(_: IdtyId) -> Result<(), DispatchError> {
        Ok(())
    }
}

/// A trait defining behavior for handling new memberships and membership renewals.
pub trait OnNewMembership<IdtyId> {
    /// Called when a new membership is created.
    fn on_created(idty_index: &IdtyId);
    /// Called when a membership is renewed.
    fn on_renewed(idty_index: &IdtyId);
}

/// A trait defining operations for handling the removal of memberships.
pub trait OnRemoveMembership<IdtyId> {
    /// Called when a membership is removed.
    fn on_removed(idty_index: &IdtyId) -> Weight;
}

/// A trait defining an operation to retrieve the count of members.
pub trait MembersCount {
    /// The count of members.
    fn members_count() -> u32;
}
