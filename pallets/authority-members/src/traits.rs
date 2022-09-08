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

use super::SessionIndex;
use frame_support::pallet_prelude::Weight;

pub trait OnNewSession {
    fn on_new_session(index: SessionIndex) -> Weight;
}

impl OnNewSession for () {
    fn on_new_session(_: SessionIndex) -> Weight {
        0
    }
}

pub trait OnRemovedMember<MemberId> {
    fn on_removed_member(member_id: MemberId) -> Weight;
}

impl<MemberId> OnRemovedMember<MemberId> for () {
    fn on_removed_member(_: MemberId) -> Weight {
        0
    }
}
