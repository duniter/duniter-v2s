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

use frame_support::dispatch::UnfilteredDispatchable;
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use pallet_certification::traits::SetNextIssuableOn;
use pallet_identity::{IdtyEvent, IdtyStatus};
use sp_membership::traits::IsInPendingMemberships;
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
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        frame_system::Config
        + pallet_certification::Config<I, IdtyIndex = IdtyIndex>
        + pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_membership::Config<I, IdtyId = IdtyIndex>
    {
        type FirstIssuableOn: Get<Self::BlockNumber>;
        type IsSubWot: Get<bool>;
        type MinCertForMembership: Get<u32>;
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
            if !T::IsSubWot::get() {
                if let Err(e) = idty_call.dispatch_bypass_filter(RawOrigin::Root.into()) {
                    sp_std::if_std! {
                        println!("fail to dispatch idty call: {:?}", e)
                    }
                    return false;
                }
            }
            true
        }
    }
}

impl<AccountId, T: Config<I>, I: 'static> pallet_identity::traits::EnsureIdtyCallAllowed<T>
    for Pallet<T, I>
where
    T: frame_system::Config<AccountId = AccountId>
        + pallet_membership::Config<I, MetaData = MembershipMetaData<AccountId>>,
{
    fn can_create_identity(creator: IdtyIndex) -> bool {
        let cert_meta = pallet_certification::Pallet::<T, I>::idty_cert_meta(creator);
        cert_meta.received_count >= T::MinCertForCreateIdtyRight::get()
            && cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number()
            && cert_meta.issued_count < T::MaxByIssuer::get()
    }
    fn can_confirm_identity(idty_index: IdtyIndex, owner_key: AccountId) -> bool {
        pallet_membership::Pallet::<T, I>::force_request_membership(
            RawOrigin::Root.into(),
            idty_index,
            MembershipMetaData(owner_key),
        )
        .is_ok()
    }
    fn can_validate_identity(idty_index: IdtyIndex) -> bool {
        pallet_membership::Pallet::<T, I>::claim_membership(
            RawOrigin::Root.into(),
            Some(idty_index),
        )
        .is_ok()
    }
}

impl<T: Config<I>, I: 'static> pallet_certification::traits::IsCertAllowed<IdtyIndex>
    for Pallet<T, I>
{
    fn is_cert_allowed(issuer: IdtyIndex, receiver: IdtyIndex) -> bool {
        if let Some(issuer_data) = pallet_identity::Pallet::<T>::identity(issuer) {
            if issuer_data.status != IdtyStatus::Validated {
                return false;
            }
            if let Some(receiver_data) = pallet_identity::Pallet::<T>::identity(receiver) {
                match receiver_data.status {
                    IdtyStatus::ConfirmedByOwner => true,
                    IdtyStatus::Created => false,
                    IdtyStatus::Validated => {
                        pallet_membership::Pallet::<T, I>::is_member(&receiver)
                            || pallet_membership::Pallet::<T, I>::is_in_pending_memberships(
                                receiver,
                            )
                    }
                }
            } else {
                // Receiver not found
                false
            }
        } else {
            // Issuer not found
            false
        }
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
            T::IsSubWot::get() && idty_value.status == IdtyStatus::Validated
        } else {
            false
        }
    }
}

impl<T: Config<I>, I: 'static, MetaData> sp_membership::traits::OnEvent<IdtyIndex, MetaData>
    for Pallet<T, I>
where
    T: pallet_membership::Config<I, MetaData = MetaData>,
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex, MetaData>) -> Weight {
        match membership_event {
            sp_membership::Event::<IdtyIndex, MetaData>::MembershipAcquired(_, _) => {}
            // Membership expiration cases:
            // Triggered by the membership pallet: we should remove the identity only for the main
            // wot
            sp_membership::Event::<IdtyIndex, MetaData>::MembershipExpired(idty_index) => {
                if !T::IsSubWot::get() {
                    Self::dispath_idty_call(pallet_identity::Call::remove_identity {
                        idty_index: *idty_index,
                        idty_name: None,
                    });
                }
            }
            // Membership revocation cases:
            // - Triggered by identity removal: the identity underlying will by removed by the
            // caller.
            // - Triggered by the membership pallet: it's ondly possible for the sub-wot, so we
            // should not remove the underlying identity
            // So, in any case, we must do nothing
            sp_membership::Event::<IdtyIndex, MetaData>::MembershipRevoked(_) => {}
            sp_membership::Event::<IdtyIndex, MetaData>::MembershipRenewed(_) => {}
            sp_membership::Event::<IdtyIndex, MetaData>::MembershipRequested(idty_index) => {
                let idty_cert_meta =
                    pallet_certification::Pallet::<T, I>::idty_cert_meta(idty_index);
                let received_count = idty_cert_meta.received_count;

                // TODO insert `receiver` in distance queue if received_count >= MinCertForMembership
                if received_count >= T::MinCertForMembership::get() as u32 {
                    // TODO insert `receiver` in distance queue
                    if Self::dispath_idty_call(pallet_identity::Call::validate_identity {
                        idty_index: *idty_index,
                    }) && received_count == T::MinReceivedCertToBeAbleToIssueCert::get()
                    {
                        Self::do_apply_first_issuable_on(*idty_index);
                    }
                }
            }
            sp_membership::Event::<IdtyIndex, MetaData>::PendingMembershipExpired(idty_index) => {
                Self::dispath_idty_call(pallet_identity::Call::remove_identity {
                    idty_index: *idty_index,
                    idty_name: None,
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
                if let Err(e) = <pallet_certification::Pallet<T, I>>::force_add_cert(
                    frame_system::Origin::<T>::Root.into(),
                    creator,
                    idty_index,
                    true,
                ) {
                    sp_std::if_std! {
                        println!("fail to force add cert: {:?}", e)
                    }
                }
            }
            IdtyEvent::Confirmed => {}
            IdtyEvent::Validated => {}
            IdtyEvent::Removed { status } => {
                if status != IdtyStatus::Validated {
                    if let Err(e) =
                        <pallet_certification::Pallet<T, I>>::remove_all_certs_received_by(
                            frame_system::Origin::<T>::Root.into(),
                            idty_index,
                        )
                    {
                        sp_std::if_std! {
                            println!("fail to remove certs received by some idty: {:?}", e)
                        }
                    }
                }
            }
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
        } else if pallet_membership::Pallet::<T, I>::is_in_pending_memberships(receiver)
            && receiver_received_count >= T::MinCertForMembership::get()
        {
            if T::IsSubWot::get() {
                if let Err(e) = pallet_membership::Pallet::<T, I>::claim_membership(
                    RawOrigin::Root.into(),
                    Some(receiver),
                ) {
                    sp_std::if_std! {
                        println!("fail to claim membership: {:?}", e)
                    }
                }
            } else {
                // TODO insert `receiver` in distance queue
                Self::dispath_idty_call(pallet_identity::Call::validate_identity {
                    idty_index: receiver,
                });
            }

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
        if receiver_received_count < T::MinCertForMembership::get()
            && pallet_membership::Pallet::<T, I>::is_member(&receiver)
        {
            if T::IsSubWot::get() {
                // Revoke receiver membership
                let call = pallet_membership::Call::<T, I>::revoke_membership {
                    maybe_idty_id: Some(receiver),
                };
                if let Err(e) = call.dispatch_bypass_filter(RawOrigin::Root.into()) {
                    sp_std::if_std! {
                        println!("fail to dispatch membership call: {:?}", e)
                    }
                }
            } else {
                // Revoke receiver membership and disable his identity
                Self::dispath_idty_call(pallet_identity::Call::remove_identity {
                    idty_index: receiver,
                    idty_name: None,
                });
            }
        }
        0
    }
}
