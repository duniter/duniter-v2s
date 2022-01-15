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

pub trait IsIdtyAllowedToCreateCert<IdtyIndex> {
    fn is_idty_allowed_to_create_cert(idty_index: IdtyIndex) -> bool;
}

pub trait OnNewcert<IdtyIndex> {
    fn on_new_cert(
        issuer: IdtyIndex,
        issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    ) -> frame_support::dispatch::Weight;
}
impl<IdtyIndex> OnNewcert<IdtyIndex> for () {
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        _receiver: IdtyIndex,
        _receiver_received_count: u32,
    ) -> frame_support::dispatch::Weight {
        0
    }
}

pub trait OnRemovedCert<IdtyIndex> {
    fn on_removed_cert(
        issuer: IdtyIndex,
        issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        expiration: bool,
    ) -> frame_support::dispatch::Weight;
}
impl<IdtyIndex> OnRemovedCert<IdtyIndex> for () {
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        _receiver: IdtyIndex,
        _receiver_received_count: u32,
        _expiration: bool,
    ) -> frame_support::dispatch::Weight {
        0
    }
}

pub trait SetNextIssuableOn<BlockNumber, IdtyIndex> {
    fn set_next_issuable_on(
        idty_index: IdtyIndex,
        next_issuable_on: BlockNumber,
    ) -> frame_support::dispatch::Weight;
}
