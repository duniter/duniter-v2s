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

use crate::{Config, IdtyIndex};
use frame_support::pallet_prelude::*;
use pallet_identity::IdtyStatus;
use sp_membership::traits::IsInPendingMemberships;
use sp_runtime::traits::IsMember;

pub struct AddCertOrigin<T, I>(core::marker::PhantomData<(T, I)>);
impl<T: Config<I>, I: 'static> EnsureOrigin<(T::Origin, IdtyIndex, IdtyIndex)>
    for AddCertOrigin<T, I>
{
    type Success = ();

    fn try_origin(
        o: (T::Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (T::Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            Ok(frame_system::RawOrigin::Signed(who)) => {
                if let Some(issuer) = pallet_identity::Pallet::<T>::identity(o.1) {
                    if who == issuer.owner_key {
                        if let Some(receiver) = pallet_identity::Pallet::<T>::identity(o.2) {
                            match receiver.status {
                                IdtyStatus::ConfirmedByOwner => Ok(()),
                                IdtyStatus::Created => Err(o),
                                IdtyStatus::Disabled => {
                                    if pallet_membership::Pallet::<T, I>::is_in_pending_memberships(o.2)
                                    {
                                        Ok(())
                                    } else {
                                        Err(o)
                                    }
                                }
                                IdtyStatus::Validated => {
                                    if pallet_membership::Pallet::<T, I>::is_member(&o.2)
                                        || pallet_membership::Pallet::<T, I>::is_in_pending_memberships(
                                            o.2,
                                        )
                                    {
                                        Ok(())
                                    } else {
                                        Err(o)
                                    }
                                }
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
                    // Issuer not found
                    Err(o)
                }
            }
            _ => Err(o),
        }
    }
}

pub struct DelCertOrigin<T, I>(core::marker::PhantomData<(T, I)>);
impl<T: Config<I>, I: 'static> EnsureOrigin<(T::Origin, IdtyIndex, IdtyIndex)>
    for DelCertOrigin<T, I>
{
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
