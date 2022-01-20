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

use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use pallet_certification::traits::SetNextIssuableOn;
use pallet_identity::{IdtyEvent, IdtyStatus};
use sp_runtime::traits::IsMember;

type IdtyIndex = u32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::UnfilteredDispatchable;
    use frame_support::traits::StorageVersion;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        frame_system::Config
        + pallet_certification::Config<I, IdtyIndex = IdtyIndex>
        + pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_membership::Config<I, IdtyId = IdtyIndex, MetaData = ()>
    {
        type FirstIssuableOn: Get<Self::BlockNumber>;
        type ManageIdentitiesChanges: Get<bool>;
        type MinCertForUdRight: Get<u32>;
        type MinCertForCreateIdtyRight: Get<u32>;
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        pub(super) fn do_apply_first_issuable_on(idty_index: IdtyIndex) {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            pallet_certification::Pallet::<T, I>::set_next_issuable_on(
                idty_index,
                block_number + T::FirstIssuableOn::get(),
            );
        }
        pub(super) fn dispath_idty_call(idty_call: pallet_identity::Call<T>) -> bool {
            if T::ManageIdentitiesChanges::get() {
                if let Err(e) = idty_call.dispatch_bypass_filter(RawOrigin::Root.into()) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                    return false;
                }
            }
            true
        }
    }
}

impl<T: Config<I>, I: 'static> pallet_identity::traits::EnsureIdtyCallAllowed<T> for Pallet<T, I> {
    fn can_create_identity(creator: IdtyIndex) -> bool {
        if let Some(cert_meta) = pallet_certification::Pallet::<T, I>::idty_cert_meta(creator) {
            cert_meta.received_count >= T::MinCertForCreateIdtyRight::get()
                && cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number()
                && cert_meta.issued_count < T::MaxByIssuer::get()
        } else {
            false
        }
    }
    fn can_confirm_identity(idty_index: IdtyIndex) -> bool {
        pallet_membership::Pallet::<T, I>::request_membership(
            RawOrigin::Root.into(),
            idty_index,
            (),
        )
        .is_ok()
    }
    fn can_validate_identity(idty_index: IdtyIndex) -> bool {
        pallet_membership::Pallet::<T, I>::claim_membership(RawOrigin::Root.into(), idty_index)
            .is_ok()
    }
}

impl<T: Config<I>, I: 'static> sp_membership::traits::IsIdtyAllowedToClaimMembership<IdtyIndex>
    for Pallet<T, I>
{
    fn is_idty_allowed_to_claim_membership(_: &IdtyIndex) -> bool {
        false
    }
}

impl<T: Config<I>, I: 'static> sp_membership::traits::IsIdtyAllowedToRenewMembership<IdtyIndex>
    for Pallet<T, I>
{
    fn is_idty_allowed_to_renew_membership(idty_index: &IdtyIndex) -> bool {
        if let Some(idty_value) = pallet_identity::Pallet::<T>::identity(idty_index) {
            idty_value.status == IdtyStatus::Validated
        } else {
            false
        }
    }
}

impl<T: Config<I>, I: 'static> sp_membership::traits::IsIdtyAllowedToRequestMembership<IdtyIndex>
    for Pallet<T, I>
{
    fn is_idty_allowed_to_request_membership(idty_index: &IdtyIndex) -> bool {
        if let Some(idty_value) = pallet_identity::Pallet::<T>::identity(idty_index) {
            idty_value.status == IdtyStatus::Disabled
        } else {
            false
        }
    }
}

impl<T: Config<I>, I: 'static> sp_membership::traits::IsOriginAllowedToUseIdty<T::Origin, IdtyIndex>
    for Pallet<T, I>
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

impl<T: Config<I>, I: 'static> sp_membership::traits::OnEvent<IdtyIndex, ()> for Pallet<T, I> {
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex>) -> Weight {
        match membership_event {
            sp_membership::Event::<IdtyIndex>::MembershipAcquired(_) => {}
            sp_membership::Event::<IdtyIndex>::MembershipExpired(idty_index) => {
                Self::dispath_idty_call(pallet_identity::Call::disable_identity {
                    idty_index: *idty_index,
                });
            }
            sp_membership::Event::<IdtyIndex>::MembershipRenewed(_) => {}
            sp_membership::Event::<IdtyIndex>::MembershipRequested(idty_index, _) => {
                if let Some(idty_cert_meta) =
                    pallet_certification::Pallet::<T, I>::idty_cert_meta(idty_index)
                {
                    let received_count = idty_cert_meta.received_count;

                    // TODO insert `receiver` in distance queue if received_count >= MinCertForUdRight
                    if received_count >= T::MinCertForUdRight::get() as u32 {
                        // TODO insert `receiver` in distance queue
                        if Self::dispath_idty_call(pallet_identity::Call::validate_identity {
                            idty_index: *idty_index,
                        }) && received_count == T::MinReceivedCertToBeAbleToIssueCert::get()
                        {
                            Self::do_apply_first_issuable_on(*idty_index);
                        }
                    }
                }
            }
            sp_membership::Event::<IdtyIndex>::MembershipRevoked(_) => {}
            sp_membership::Event::<IdtyIndex>::PendingMembershipExpired(idty_index) => {
                Self::dispath_idty_call(pallet_identity::Call::remove_identity {
                    idty_index: *idty_index,
                });
            }
        }
        0
    }
}

impl<T: Config<I>, I: 'static> pallet_identity::traits::OnIdtyChange<T> for Pallet<T, I> {
    fn on_idty_change(idty_index: IdtyIndex, idty_event: IdtyEvent<T>) -> Weight {
        match idty_event {
            IdtyEvent::Created { creator } => {
                if let Err(e) = <pallet_certification::Pallet<T, I>>::add_cert(
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

impl<T: Config<I>, I: 'static> pallet_certification::traits::OnNewcert<IdtyIndex> for Pallet<T, I> {
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    ) -> Weight {
        if pallet_membership::Pallet::<T, I>::is_member(&receiver) {
            if receiver_received_count == T::MinReceivedCertToBeAbleToIssueCert::get() {
                Self::do_apply_first_issuable_on(receiver);
            }
        } else if pallet_membership::Pallet::<T, I>::pending_membership(receiver).is_some()
            && receiver_received_count >= T::MinCertForUdRight::get()
        {
            // TODO insert `receiver` in distance queue
            Self::dispath_idty_call(pallet_identity::Call::validate_identity {
                idty_index: receiver,
            });

            if receiver_received_count == T::MinReceivedCertToBeAbleToIssueCert::get() {
                Self::do_apply_first_issuable_on(receiver);
            }
        }
        0
    }
}

impl<T: Config<I>, I: 'static> pallet_certification::traits::OnRemovedCert<IdtyIndex>
    for Pallet<T, I>
{
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        _expiration: bool,
    ) -> Weight {
        if receiver_received_count < T::MinCertForUdRight::get() {
            // Revoke receiver membership and disable his identity
            if let Err(e) = pallet_membership::Pallet::<T, I>::revoke_membership(
                RawOrigin::Root.into(),
                receiver,
            ) {
                sp_std::if_std! {
                    println!("{:?}", e)
                }
            } else {
                Self::dispath_idty_call(pallet_identity::Call::disable_identity {
                    idty_index: receiver,
                });
            }
        }
        0
    }
}
