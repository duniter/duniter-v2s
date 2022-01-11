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

use crate::entities::{IdtyData, IdtyRight};
use crate::{BlockNumber, IdtyIndex};
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::EnsureOrigin;
use pallet_certification::traits::IsIdtyAllowedToCreateCert;
use pallet_identity::IdtyStatus;

pub struct EnsureIdtyCallAllowedImpl<Runtime, IsIdtyAllowedToCreateCertImpl>(
    core::marker::PhantomData<(Runtime, IsIdtyAllowedToCreateCertImpl)>,
);
impl<
        Runtime: frame_system::Config<BlockNumber = BlockNumber>
            + pallet_identity::Config<
                IdtyData = IdtyData,
                IdtyIndex = IdtyIndex,
                IdtyRight = IdtyRight,
            >,
        IsIdtyAllowedToCreateCertImpl: IsIdtyAllowedToCreateCert<IdtyIndex>,
    > pallet_identity::traits::EnsureIdtyCallAllowed<Runtime>
    for EnsureIdtyCallAllowedImpl<Runtime, IsIdtyAllowedToCreateCertImpl>
{
    fn can_create_identity(
        origin: Runtime::Origin,
        creator: IdtyIndex,
        _idty_name: &pallet_identity::IdtyName,
        _idty_owner_key: &Runtime::AccountId,
    ) -> Result<(), DispatchError> {
        match origin.into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            Ok(frame_system::RawOrigin::Signed(signer)) => {
                if let Some(creator_idty) = pallet_identity::Pallet::<Runtime>::identity(creator) {
                    if let Some(authorized_key) = creator_idty.get_right_key(IdtyRight::CreateIdty)
                    {
                        if signer != authorized_key {
                            frame_support::runtime_print!("signer != authorized_key");
                            Err(DispatchError::Other("signer != authorized_key"))
                        } else if !IsIdtyAllowedToCreateCertImpl::is_idty_allowed_to_create_cert(
                            creator,
                        ) {
                            frame_support::runtime_print!("not allowed to create cert");
                            Err(DispatchError::Other("not allowed to create cert"))
                        } else if creator_idty.data.can_create_on
                            > frame_system::Pallet::<Runtime>::block_number()
                        {
                            frame_support::runtime_print!("Not respect IdtyCreatePeriod");
                            Err(DispatchError::Other("Not respect IdtyCreatePeriod"))
                        } else {
                            Ok(())
                        }
                    } else {
                        frame_support::runtime_print!("Idty not have right CreateIdty");
                        Err(DispatchError::Other("Idty not have right CreateIdty"))
                    }
                } else {
                    frame_support::runtime_print!("Idty not found");
                    Err(DispatchError::Other("Idty not found"))
                }
            }
            _ => {
                frame_support::runtime_print!("Origin neither root or signed");
                Err(DispatchError::Other("Origin neither root or signed"))
            }
        }
    }
}

pub struct AddStrongCertOrigin<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex, IdtyRight = IdtyRight>>
    EnsureOrigin<(Runtime::Origin, IdtyIndex, IdtyIndex)> for AddStrongCertOrigin<Runtime>
{
    type Success = ();

    fn try_origin(
        o: (Runtime::Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (Runtime::Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            Ok(frame_system::RawOrigin::Signed(who)) => {
                if let Some(issuer) = pallet_identity::Pallet::<Runtime>::identity(o.1) {
                    if let Some(allowed_key) = issuer.get_right_key(IdtyRight::StrongCert) {
                        if who == allowed_key {
                            if let Some(receiver) =
                                pallet_identity::Pallet::<Runtime>::identity(o.2)
                            {
                                match receiver.status {
                                    IdtyStatus::ConfirmedByOwner | IdtyStatus::Validated => Ok(()),
                                    IdtyStatus::Created | IdtyStatus::Expired => Err(o),
                                }
                            } else {
                                // Receiver not found
                                Err(o)
                            }
                        } else {
                            // Bad key
                            Err(o)
                        }
                    } else {
                        // Issuer has not right StrongCert
                        Err(o)
                    }
                } else {
                    // Issuer not found
                    Err(o)
                }
            }
            _ => Err(o),
        }
    }
}

pub struct DelStrongCertOrigin<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime: frame_system::Config> EnsureOrigin<(Runtime::Origin, IdtyIndex, IdtyIndex)>
    for DelStrongCertOrigin<Runtime>
{
    type Success = ();

    fn try_origin(
        o: (Runtime::Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (Runtime::Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::Origin::<Runtime>::Root) => Ok(()),
            _ => Err(o),
        }
    }
}
