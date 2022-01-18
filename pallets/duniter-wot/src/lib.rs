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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

//pub mod traits;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/*#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

pub use pallet::*;
pub use types::*;

use frame_support::instances::Instance1;
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use pallet_certification::traits::SetNextIssuableOn;
use pallet_identity::{IdtyEvent, IdtyStatus};
use sp_runtime::traits::IsMember;

type IdtyIndex = u32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_certification::Config<Instance1, IdtyIndex = IdtyIndex>
        + pallet_identity::Config<IdtyIndex = IdtyIndex, IdtyRight = IdtyRight>
        + pallet_membership::Config<Instance1, IdtyId = IdtyIndex>
    {
        type MinCertForUdRight: Get<u8>;
        type MinCertForCertRight: Get<u8>;
        type MinCertForCreateIdtyRight: Get<u8>;
        type FirstIssuableOn: Get<Self::BlockNumber>;
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub(super) fn do_apply_first_issuable_on(idty_index: IdtyIndex) {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            pallet_certification::Pallet::<T, Instance1>::set_next_issuable_on(
                idty_index,
                block_number + T::FirstIssuableOn::get(),
            );
        }
        pub(super) fn do_add_cert_right(idty_index: IdtyIndex) {
            match pallet_identity::Pallet::<T>::add_right(
                RawOrigin::Root.into(),
                idty_index,
                IdtyRight::StrongCert,
            ) {
                Ok(_) => {
                    Self::do_apply_first_issuable_on(idty_index);
                }
                Err(e) => {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
            }
        }
        pub(super) fn do_add_rights(idty_index: IdtyIndex, received_cert_count: u32) {
            if received_cert_count >= T::MinCertForUdRight::get() as u32 {
                if let Err(e) = pallet_identity::Pallet::<T>::add_right(
                    RawOrigin::Root.into(),
                    idty_index,
                    IdtyRight::Ud,
                ) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
            }
            if received_cert_count >= T::MinCertForCertRight::get() as u32 {
                Self::do_add_cert_right(idty_index);
            }
            if received_cert_count >= T::MinCertForCreateIdtyRight::get() as u32 {
                if let Err(e) = pallet_identity::Pallet::<T>::add_right(
                    RawOrigin::Root.into(),
                    idty_index,
                    IdtyRight::CreateIdty,
                ) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
            }
        }
    }
}

impl<T: Config> pallet_identity::traits::EnsureIdtyCallAllowed<T> for Pallet<T> {
    fn can_create_identity(creator: IdtyIndex) -> bool {
        if let Some(cert_meta) =
            pallet_certification::Pallet::<T, Instance1>::idty_cert_meta(creator)
        {
            use frame_support::traits::Get as _;
            cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number()
                && cert_meta.issued_count < T::MaxByIssuer::get()
        } else {
            true
        }
    }
    fn can_confirm_identity(idty_index: IdtyIndex) -> bool {
        pallet_membership::Pallet::<T, Instance1>::request_membership(
            RawOrigin::Root.into(),
            idty_index,
        )
        .is_ok()
    }
    fn can_validate_identity(idty_index: IdtyIndex) -> bool {
        pallet_membership::Pallet::<T, Instance1>::claim_membership(
            RawOrigin::Root.into(),
            idty_index,
        )
        .is_ok()
    }
}

impl<T: Config> sp_membership::traits::IsIdtyAllowedToClaimMembership<IdtyIndex> for Pallet<T> {
    fn is_idty_allowed_to_claim_membership(_: &IdtyIndex) -> bool {
        false
    }
}

impl<T: Config> sp_membership::traits::IsIdtyAllowedToRenewMembership<IdtyIndex> for Pallet<T> {
    fn is_idty_allowed_to_renew_membership(idty_index: &IdtyIndex) -> bool {
        if let Some(idty_value) = pallet_identity::Pallet::<T>::identity(idty_index) {
            idty_value.status == IdtyStatus::Validated
        } else {
            false
        }
    }
}

impl<T: Config> sp_membership::traits::IsIdtyAllowedToRequestMembership<IdtyIndex> for Pallet<T> {
    fn is_idty_allowed_to_request_membership(idty_index: &IdtyIndex) -> bool {
        if let Some(idty_value) = pallet_identity::Pallet::<T>::identity(idty_index) {
            idty_value.status == IdtyStatus::Validated
        } else {
            false
        }
    }
}

impl<T: Config> sp_membership::traits::IsOriginAllowedToUseIdty<T::Origin, IdtyIndex>
    for Pallet<T>
{
    fn is_origin_allowed_to_use_idty(
        origin: &T::Origin,
        idty_index: &IdtyIndex,
    ) -> sp_membership::OriginPermission {
        match origin.clone().into() {
            Ok(RawOrigin::Root) => sp_membership::OriginPermission::Root,
            Ok(RawOrigin::Signed(account_id)) => {
                if let Some(idty_val) = pallet_identity::Pallet::<T>::identity(idty_index) {
                    if account_id == idty_val.owner_key {
                        sp_membership::OriginPermission::Allowed
                    } else {
                        sp_membership::OriginPermission::Forbidden
                    }
                } else {
                    sp_membership::OriginPermission::Forbidden
                }
            }
            _ => sp_membership::OriginPermission::Forbidden,
        }
    }
}

impl<T: crate::pallet::Config> sp_membership::traits::OnEvent<IdtyIndex> for Pallet<T> {
    fn on_event(membership_event: sp_membership::Event<IdtyIndex>) -> Weight {
        match membership_event {
            sp_membership::Event::<IdtyIndex>::MembershipAcquired(_) => {}
            sp_membership::Event::<IdtyIndex>::MembershipExpired(idty_index) => {
                if let Err(e) = pallet_identity::Pallet::<T>::remove_all_rights(
                    RawOrigin::Root.into(),
                    idty_index,
                ) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
            }
            sp_membership::Event::<IdtyIndex>::MembershipRenewed(_) => {}
            sp_membership::Event::<IdtyIndex>::MembershipRequested(idty_index) => {
                if let Some(idty_cert_meta) =
                    pallet_certification::Pallet::<T, Instance1>::idty_cert_meta(idty_index)
                {
                    let received_count = idty_cert_meta.received_count;

                    // TODO insert `receiver` in distance queue if received_count >= MinCertForUdRight
                    Self::do_add_rights(idty_index, received_count);
                }
            }
            sp_membership::Event::<IdtyIndex>::MembershipRevoked(_) => {}
            sp_membership::Event::<IdtyIndex>::PendingMembershipExpired(idty_index) => {
                if let Err(e) = pallet_identity::Pallet::<T>::remove_identity(
                    RawOrigin::Root.into(),
                    idty_index,
                ) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
            }
        }
        0
    }
}

impl<T: Config> pallet_identity::traits::OnIdtyChange<T> for Pallet<T> {
    fn on_idty_change(idty_index: IdtyIndex, idty_event: IdtyEvent<T>) -> Weight {
        match idty_event {
            IdtyEvent::Created { creator } => {
                if let Err(e) = <pallet_certification::Pallet<T, Instance1>>::add_cert(
                    frame_system::Origin::<T>::Root.into(),
                    creator,
                    idty_index,
                ) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
            }
            IdtyEvent::Confirmed => {}
            IdtyEvent::Validated => {}
            IdtyEvent::Removed => {}
        }
        0
    }
}

impl<T: crate::pallet::Config> pallet_certification::traits::OnNewcert<IdtyIndex> for Pallet<T> {
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    ) -> Weight {
        if pallet_membership::Pallet::<T, Instance1>::is_member(&receiver) {
            Self::do_add_rights(receiver, receiver_received_count);
        } else if pallet_membership::Pallet::<T, Instance1>::pending_membership(receiver).is_some()
            && receiver_received_count >= T::MinCertForUdRight::get() as u32
        {
            // TODO insert `receiver` in distance queue
            let mut rights = sp_std::vec![IdtyRight::Ud];
            let mut cert_right = false;
            if receiver_received_count >= T::MinCertForCertRight::get() as u32 {
                rights.push(IdtyRight::StrongCert);
                cert_right = true;
            }
            if receiver_received_count >= T::MinCertForCreateIdtyRight::get() as u32 {
                rights.push(IdtyRight::CreateIdty);
            }
            if let Err(e) = pallet_identity::Pallet::<T>::validate_identity(
                RawOrigin::Root.into(),
                receiver,
                rights,
            ) {
                sp_std::if_std! {
                    println!("{:?}", e)
                }
                return 0;
            }

            if cert_right {
                Self::do_apply_first_issuable_on(receiver);
            }
        }
        0
    }
}

impl<T: crate::pallet::Config> pallet_certification::traits::OnRemovedCert<IdtyIndex>
    for Pallet<T>
{
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        _expiration: bool,
    ) -> Weight {
        if receiver_received_count < T::MinCertForUdRight::get() as u32 {
            if let Err(e) = pallet_identity::Pallet::<T>::del_right(
                RawOrigin::Root.into(),
                receiver,
                IdtyRight::Ud,
            ) {
                sp_std::if_std! {
                    println!("{:?}", e)
                }
            }
        }
        if receiver_received_count < T::MinCertForCertRight::get() as u32 {
            if let Err(e) = pallet_identity::Pallet::<T>::del_right(
                RawOrigin::Root.into(),
                receiver,
                IdtyRight::StrongCert,
            ) {
                sp_std::if_std! {
                    println!("{:?}", e)
                }
            }
        }
        if receiver_received_count < T::MinCertForCreateIdtyRight::get() as u32 {
            if let Err(e) = pallet_identity::Pallet::<T>::del_right(
                RawOrigin::Root.into(),
                receiver,
                IdtyRight::CreateIdty,
            ) {
                sp_std::if_std! {
                    println!("{:?}", e)
                }
            }
        }
        0
    }
}
