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

pub trait OnNewSession {
    fn on_new_session(index: SessionIndex);
}

impl OnNewSession for () {
    fn on_new_session(_: SessionIndex) {}
}

/// Handle the consequences of going in the authority set for other pallets.
/// Typically, a smith won't expire as long as he is in the authority set.
pub trait OnIncomingMember<MemberId> {
    fn on_incoming_member(member_id: MemberId);
}
/// By default: no consequences
impl<MemberId> OnIncomingMember<MemberId> for () {
    fn on_incoming_member(_: MemberId) {}
}

/// Handle the consequences of going out of authority set for other pallets.
/// Typically, the smiths are not allowed to stay offline for a too long time.
pub trait OnOutgoingMember<MemberId> {
    fn on_outgoing_member(member_id: MemberId);
}
/// By default: no consequences
impl<MemberId> OnOutgoingMember<MemberId> for () {
    fn on_outgoing_member(_: MemberId) {}
}
