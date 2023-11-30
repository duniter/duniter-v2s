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

pub trait CheckCertAllowed<IdtyIndex> {
    fn check_cert_allowed(issuer: IdtyIndex, receiver: IdtyIndex) -> Result<(), DispatchError>;
}

impl<IdtyIndex> CheckCertAllowed<IdtyIndex> for () {
    fn check_cert_allowed(_issuer: IdtyIndex, _receiver: IdtyIndex) -> Result<(), DispatchError> {
        Ok(())
    }
}

pub trait OnNewcert<IdtyIndex> {
    fn on_new_cert(
        issuer: IdtyIndex,
        issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    );
}
impl<IdtyIndex> OnNewcert<IdtyIndex> for () {
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        _receiver: IdtyIndex,
        _receiver_received_count: u32,
    ) {
    }
}

pub trait OnRemovedCert<IdtyIndex> {
    fn on_removed_cert(
        issuer: IdtyIndex,
        issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        expiration: bool,
    );
}
impl<IdtyIndex> OnRemovedCert<IdtyIndex> for () {
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        _receiver: IdtyIndex,
        _receiver_received_count: u32,
        _expiration: bool,
    ) {
    }
}

pub trait SetNextIssuableOn<BlockNumber, IdtyIndex> {
    fn set_next_issuable_on(idty_index: IdtyIndex, next_issuable_on: BlockNumber);
}
