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

use frame_support::pallet_prelude::Weight;

pub trait OnRemovedMember<MemberId> {
    fn on_removed_member(member_id: MemberId) -> Weight;
}

impl<MemberId> OnRemovedMember<MemberId> for () {
    fn on_removed_member(_: MemberId) -> Weight {
        0
    }
}
