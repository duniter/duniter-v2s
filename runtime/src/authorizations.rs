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

use crate::{
    AccountId, BlockNumber, Identity, IdtyData, IdtyDid, IdtyIndex, IdtyRight, Origin, Runtime,
    StrongCert, System,
};
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::EnsureOrigin;
use pallet_identity::IdtyStatus;

const IDTY_CREATE_PERIOD: BlockNumber = 100;

pub struct EnsureIdtyCallAllowedImpl;
impl pallet_identity::traits::EnsureIdtyCallAllowed<Runtime> for EnsureIdtyCallAllowedImpl {
    fn create_identity(
        origin: Origin,
        creator: IdtyIndex,
        _idty_did: &IdtyDid,
        _idty_owner_key: &AccountId,
    ) -> Result<IdtyData, DispatchError> {
        let block_number = System::block_number();
        let creator_idty_data = IdtyData {
            can_create_on: block_number + IDTY_CREATE_PERIOD,
        };
        let new_idty_data = IdtyData { can_create_on: 0 };
        match origin.into() {
            Ok(frame_system::RawOrigin::Root) => {
                Identity::set_idty_data(creator, creator_idty_data);
                Ok(new_idty_data)
            }
            Ok(frame_system::RawOrigin::Signed(signer)) => {
                let creator_idty = Identity::identity(creator);

                if let Some(authorized_key) = creator_idty.get_right_key(IdtyRight::CreateIdty) {
                    if signer != authorized_key {
                        frame_support::runtime_print!("signer != authorized_key");
                        Err(DispatchError::Other("signer != authorized_key"))
                    } else if !StrongCert::is_idty_allowed_to_create_cert(creator) {
                        frame_support::runtime_print!("not allowed to create cert");
                        Err(DispatchError::Other("not allowed to create cert"))
                    } else if creator_idty.data.can_create_on > System::block_number() {
                        frame_support::runtime_print!("Not respect IdtyCreatePeriod");
                        Err(DispatchError::Other("Not respect IdtyCreatePeriod"))
                    } else {
                        Identity::set_idty_data(creator, creator_idty_data);
                        Ok(new_idty_data)
                    }
                } else {
                    frame_support::runtime_print!("Idty not have right CreateIdty");
                    Err(DispatchError::Other("Idty not have right CreateIdty"))
                }
            }
            _ => {
                frame_support::runtime_print!("Origin neither root or signed");
                Err(DispatchError::Other("Origin neither root or signed"))
            }
        }
    }
}

pub struct AddStrongCertOrigin;
impl EnsureOrigin<(Origin, IdtyIndex, IdtyIndex)> for AddStrongCertOrigin {
    type Success = ();

    fn try_origin(
        o: (Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            Ok(frame_system::RawOrigin::Signed(who)) => {
                let issuer = Identity::identity(o.1);
                if let Some(allowed_key) = issuer.get_right_key(IdtyRight::StrongCert) {
                    if who == allowed_key {
                        let receiver = Identity::identity(o.2);
                        match receiver.status {
                            IdtyStatus::ConfirmedByOwner | IdtyStatus::Validated => Ok(()),
                            IdtyStatus::Created | IdtyStatus::Expired => Err(o),
                        }
                    } else {
                        // Bad key
                        Err(o)
                    }
                } else {
                    // Issuer has not right StrongCert
                    Err(o)
                }
            }
            _ => Err(o),
        }
    }
}

pub struct DelStrongCertOrigin;
impl EnsureOrigin<(Origin, IdtyIndex, IdtyIndex)> for DelStrongCertOrigin {
    type Success = ();

    fn try_origin(
        o: (Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            _ => Err(o),
        }
    }
}
