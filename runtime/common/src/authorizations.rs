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

use crate::entities::IdtyRight;
use crate::{BlockNumber, IdtyIndex};
use frame_support::traits::EnsureOrigin;
use pallet_certification::traits::IsIdtyAllowedToCreateCert;
use pallet_identity::IdtyStatus;

pub struct EnsureIdtyCallAllowedImpl<Runtime, IsIdtyAllowedToCreateCertImpl>(
    core::marker::PhantomData<(Runtime, IsIdtyAllowedToCreateCertImpl)>,
);
impl<
        Runtime: frame_system::Config<BlockNumber = BlockNumber>
            + pallet_identity::Config<IdtyIndex = IdtyIndex>,
        IsIdtyAllowedToCreateCertImpl: IsIdtyAllowedToCreateCert<IdtyIndex>,
    > pallet_identity::traits::EnsureIdtyCallAllowed<Runtime>
    for EnsureIdtyCallAllowedImpl<Runtime, IsIdtyAllowedToCreateCertImpl>
{
    fn can_create_identity(creator: IdtyIndex) -> bool {
        IsIdtyAllowedToCreateCertImpl::is_idty_allowed_to_create_cert(creator)
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
