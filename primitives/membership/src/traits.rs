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

pub trait CheckMembershipCallAllowed<IdtyId> {
    fn check_idty_allowed_to_claim_membership(idty_id: &IdtyId) -> Result<(), DispatchError>;
    fn check_idty_allowed_to_renew_membership(idty_id: &IdtyId) -> Result<(), DispatchError>;
    fn check_idty_allowed_to_request_membership(idty_id: &IdtyId) -> Result<(), DispatchError>;
}

impl<IdtyId> CheckMembershipCallAllowed<IdtyId> for () {
    fn check_idty_allowed_to_claim_membership(_: &IdtyId) -> Result<(), DispatchError> {
        Ok(())
    }
    fn check_idty_allowed_to_renew_membership(_: &IdtyId) -> Result<(), DispatchError> {
        Ok(())
    }
    fn check_idty_allowed_to_request_membership(_: &IdtyId) -> Result<(), DispatchError> {
        Ok(())
    }
}

pub trait IsInPendingMemberships<IdtyId> {
    fn is_in_pending_memberships(idty_id: IdtyId) -> bool;
}

pub trait OnEvent<IdtyId> {
    fn on_event(event: &crate::Event<IdtyId>) -> Weight;
}

impl<IdtyId> OnEvent<IdtyId> for () {
    fn on_event(_: &crate::Event<IdtyId>) -> Weight {
        Weight::zero()
    }
}

pub trait MembersCount {
    fn members_count() -> u32;
}

pub trait Validate<AccountId> {
    fn validate(&self, account_id: &AccountId) -> bool;
}

impl<AccountId> Validate<AccountId> for () {
    fn validate(&self, _account_id: &AccountId) -> bool {
        true
    }
}
