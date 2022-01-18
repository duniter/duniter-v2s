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
use frame_support::pallet_prelude::{DispatchResultWithPostInfo, TypeInfo, Weight};

pub trait IsIdtyAllowedToClaimMembership<IdtyId> {
    fn is_idty_allowed_to_claim_membership(idty_id: &IdtyId) -> bool;
}

impl<IdtyId> IsIdtyAllowedToClaimMembership<IdtyId> for () {
    fn is_idty_allowed_to_claim_membership(_: &IdtyId) -> bool {
        true
    }
}

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

pub trait IsOriginAllowedToUseIdty<Origin, IdtyId> {
    fn is_origin_allowed_to_use_idty(origin: &Origin, idty_id: &IdtyId) -> OriginPermission;
}

impl<Origin, IdtyId> IsOriginAllowedToUseIdty<Origin, IdtyId> for () {
    fn is_origin_allowed_to_use_idty(_: &Origin, _: &IdtyId) -> OriginPermission {
        OriginPermission::Allowed
    }
}

pub trait IsInPendingMemberships<IdtyId> {
    fn is_in_pending_memberships(idty_id: IdtyId) -> bool;
}

pub trait OnEvent<IdtyId> {
    fn on_event(event: crate::Event<IdtyId>) -> Weight;
}

impl<IdtyId> OnEvent<IdtyId> for () {
    fn on_event(_: crate::Event<IdtyId>) -> Weight {
        0
    }
}

pub trait MembershipAction<IdtyId, Origin> {
    fn request_membership_(origin: Origin, idty_id: IdtyId) -> DispatchResultWithPostInfo;
    fn claim_membership_(origin: Origin, idty_id: IdtyId) -> DispatchResultWithPostInfo;
    fn renew_membership_(origin: Origin, idty_id: IdtyId) -> DispatchResultWithPostInfo;
    fn revoke_membership_(origin: Origin, idty_id: IdtyId) -> DispatchResultWithPostInfo;
    fn force_claim_membership(idty_id: IdtyId) -> Weight;
    fn force_renew_membership(idty_id: IdtyId) -> Weight;
    fn force_revoke_membership(idty_id: IdtyId) -> Weight;
}

impl<IdtyId, Origin> MembershipAction<IdtyId, Origin> for () {
    fn request_membership_(_: Origin, _: IdtyId) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
    fn claim_membership_(_: Origin, _: IdtyId) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
    fn renew_membership_(_: Origin, _: IdtyId) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
    fn revoke_membership_(_: Origin, _: IdtyId) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
    fn force_claim_membership(_: IdtyId) -> Weight {
        0
    }
    fn force_renew_membership(_: IdtyId) -> Weight {
        0
    }
    fn force_revoke_membership(_: IdtyId) -> Weight {
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
