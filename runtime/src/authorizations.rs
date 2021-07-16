// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use crate::{AccountId, Identity, IdtyDid, IdtyIndex, IdtyRight, Origin, Runtime};

pub struct EnsureIdtyCallAllowedImpl;
impl pallet_identity::traits::EnsureIdtyCallAllowed<Runtime> for EnsureIdtyCallAllowedImpl {
    fn create_identity(
        origin: Origin,
        creator: IdtyIndex,
        _idty_did: &IdtyDid,
        _idty_owner_key: &AccountId,
    ) -> bool {
        match origin.into() {
            Ok(frame_system::RawOrigin::Root) => true,
            Ok(frame_system::RawOrigin::Signed(signer)) => {
                let creator_idty = Identity::identity(creator);

                if let Some(authorized_key) = creator_idty.get_right_key(IdtyRight::CreateIdty) {
                    signer == authorized_key
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
