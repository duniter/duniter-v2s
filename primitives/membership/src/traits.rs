// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use crate::*;
use frame_support::pallet_prelude::{TypeInfo, Weight};

pub trait IsIdtyAllowedToRenewMembership<IdtyId> {
    fn is_idty_allowed_to_renew_membership(idty_id: &IdtyId) -> bool;
}

impl<IdtyId> IsIdtyAllowedToRenewMembership<IdtyId> for () {
    fn is_idty_allowed_to_renew_membership(_: &IdtyId) -> bool {
        true
    }
}

pub trait IsIdtyAllowedToRequestMembership<IdtyId> {
    fn is_idty_allowed_to_request_membership(idty_id: &IdtyId) -> bool;
}

impl<IdtyId> IsIdtyAllowedToRequestMembership<IdtyId> for () {
    fn is_idty_allowed_to_request_membership(_: &IdtyId) -> bool {
        true
    }
}

pub trait IsInPendingMemberships<IdtyId> {
    fn is_in_pending_memberships(idty_id: IdtyId) -> bool;
}

pub trait OnEvent<IdtyId, MetaData> {
    fn on_event(event: &crate::Event<IdtyId, MetaData>) -> Weight;
}

impl<IdtyId, MetaData> OnEvent<IdtyId, MetaData> for () {
    fn on_event(_: &crate::Event<IdtyId, MetaData>) -> Weight {
        0
    }
}

pub trait MembershipExternalStorage<BlockNumber: Decode + Encode + TypeInfo, IdtyId>:
    sp_runtime::traits::IsMember<IdtyId>
{
    fn insert(idty_id: IdtyId, membership_data: MembershipData<BlockNumber>);
    fn get(idty_id: &IdtyId) -> Option<MembershipData<BlockNumber>>;
    fn remove(idty_id: &IdtyId);
}

static INVALID_CONF_MSG: &str = "invalid pallet configuration: if `MembershipExternalStorage` = (), you must set `ExternalizeMembershipStorage` to `false`.";

pub struct NoExternalStorage;
impl<IdtyId> sp_runtime::traits::IsMember<IdtyId> for NoExternalStorage {
    fn is_member(_: &IdtyId) -> bool {
        panic!("{}", INVALID_CONF_MSG)
    }
}
impl<BlockNumber: Decode + Encode + TypeInfo, IdtyId> MembershipExternalStorage<BlockNumber, IdtyId>
    for NoExternalStorage
{
    fn insert(_: IdtyId, _: MembershipData<BlockNumber>) {
        panic!("{}", INVALID_CONF_MSG)
    }
    fn get(_: &IdtyId) -> Option<MembershipData<BlockNumber>> {
        panic!("{}", INVALID_CONF_MSG)
    }
    fn remove(_: &IdtyId) {
        panic!("{}", INVALID_CONF_MSG)
    }
}

pub trait Validate<AccountId> {
    fn validate(&self, account_id: &AccountId) -> bool;
}

impl<AccountId> Validate<AccountId> for () {
    fn validate(&self, _account_id: &AccountId) -> bool {
        true
    }
}