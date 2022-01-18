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

use pallet_identity::IdtyStatus;

use crate::{Config, IdtyIndex};
use frame_support::instances::Instance1;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::IsMember;

pub struct AddStrongCertOrigin<T>(core::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<(T::Origin, IdtyIndex, IdtyIndex)> for AddStrongCertOrigin<T> {
    type Success = ();

    fn try_origin(
        o: (T::Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (T::Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            Ok(frame_system::RawOrigin::Signed(who)) => {
                if let Some(issuer) = pallet_identity::Pallet::<T>::identity(o.1) {
                    if let Some(allowed_key) = issuer.get_right_key(IdtyRight::StrongCert) {
                        if who == allowed_key {
                            if let Some(receiver) = pallet_identity::Pallet::<T>::identity(o.2) {
                                match receiver.status {
                                    IdtyStatus::ConfirmedByOwner => Ok(()),
                                    IdtyStatus::Validated => {
                                        if pallet_membership::Pallet::<T, Instance1>::is_member(
                                            &o.2,
                                        ) || pallet_membership::Pallet::<T, Instance1>::pending_membership(&o.2).is_some() {
                                            Ok(())
                                        } else {
                                            Err(o)
                                        }
                                    }
                                    IdtyStatus::Created => Err(o),
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

pub struct DelStrongCertOrigin<T>(core::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<(T::Origin, IdtyIndex, IdtyIndex)> for DelStrongCertOrigin<T> {
    type Success = ();

    fn try_origin(
        o: (T::Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (T::Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::Origin::<T>::Root) => Ok(()),
            _ => Err(o),
        }
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub enum IdtyRight {
    CreateIdty,
    LightCert,
    StrongCert,
    Ud,
}
impl Default for IdtyRight {
    fn default() -> Self {
        Self::Ud
    }
}
impl pallet_identity::traits::IdtyRight for IdtyRight {
    fn allow_owner_key(self) -> bool {
        match self {
            Self::CreateIdty | Self::LightCert | IdtyRight::StrongCert | Self::Ud => true,
            //IdtyRight::StrongCert => false,
            //_ => false,
        }
    }
    fn create_idty_right() -> Self {
        Self::CreateIdty
    }
}
