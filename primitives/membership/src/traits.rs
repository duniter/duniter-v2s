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

pub trait CheckMembershipOpAllowed<IdtyId> {
    fn check_add_membership(idty_id: IdtyId) -> Result<(), DispatchError>;
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

pub trait OnEvent<IdtyId> {
    fn on_event(event: &crate::Event<IdtyId>);
}

// impl<IdtyId> OnEvent<IdtyId> for () {
//     fn on_event(_: &crate::Event<IdtyId>) {}
// }

pub trait MembersCount {
    fn members_count() -> u32;
}
