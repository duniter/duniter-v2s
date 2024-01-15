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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod traits;

/*#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

pub use pallet::*;

use traits::*;

use frame_support::pallet_prelude::*;
use pallet_certification::traits::SetNextIssuableOn;
use pallet_identity::{IdtyEvent, IdtyStatus};
use pallet_membership::MembershipRemovalReason;

type IdtyIndex = u32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_certification::Config<IdtyIndex = IdtyIndex>
        + pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_membership::Config<IdtyId = IdtyIndex>
    {
        /// Distance evaluation provider
        type IsDistanceOk: IsDistanceOk<IdtyIndex>;
        #[pallet::constant]
        type FirstIssuableOn: Get<Self::BlockNumber>;
        #[pallet::constant]
        type MinCertForMembership: Get<u32>;
        #[pallet::constant]
        type MinCertForCreateIdtyRight: Get<u32>;
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub(super) fn do_apply_first_issuable_on(idty_index: IdtyIndex) {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            pallet_certification::Pallet::<T>::set_next_issuable_on(
                idty_index,
                block_number + T::FirstIssuableOn::get(),
            );
        }
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient certifications received to claim membership.
        NotEnoughCertsToClaimMembership,
        /// Distance is invalid.
        DistanceIsInvalid,
        /// Distance is not evaluated.
        DistanceNotEvaluated,
        /// Distance evaluation has been requested but is still pending
        DistanceEvaluationPending,
        /// Distance evaluation has not been requested
        DistanceEvaluationNotRequested,
        /// Identity is not allowed to request membership.
        IdtyNotAllowedToRequestMembership,
        /// Identity not allowed to renew membership.
        IdtyNotAllowedToRenewMembership,
        /// Identity creation period not respected.
        IdtyCreationPeriodNotRespected,
        /// Insufficient received certifications to create identity.
        NotEnoughReceivedCertsToCreateIdty,
        /// Maximum number of emitted certifications reached.
        MaxEmittedCertsReached,
        /// Not allowed to change identity address.
        NotAllowedToChangeIdtyAddress,
        /// Not allowed to remove identity.
        NotAllowedToRemoveIdty,
        /// Issuer cannot emit a certification because it is not member.
        IssuerNotMember,
        /// Cannot issue a certification to an unconfirmed identity
        CertToUnconfirmed,
        /// Cannot issue a certification to a revoked identity
        CertToRevoked,
        /// Issuer or receiver not found.
        IdtyNotFound,
    }
}

// implement identity call checks
impl<AccountId, T: Config> pallet_identity::traits::CheckIdtyCallAllowed<T> for Pallet<T>
where
    T: frame_system::Config<AccountId = AccountId> + pallet_membership::Config,
{
    // identity creation checks
    fn check_create_identity(creator: IdtyIndex) -> Result<(), DispatchError> {
        let cert_meta = pallet_certification::Pallet::<T>::idty_cert_meta(creator);
        // perform all checks
        // 1. check that identity has the right to create an identity
        // identity can be member with 5 certifications and still not reach identity creation threshold which could be higher (6, 7...)
        ensure!(
            cert_meta.received_count >= T::MinCertForCreateIdtyRight::get(),
            Error::<T>::NotEnoughReceivedCertsToCreateIdty
        );
        // 2. check that issuer can emit one more certification
        // (this is only a partial check)
        ensure!(
            cert_meta.issued_count < T::MaxByIssuer::get(),
            Error::<T>::MaxEmittedCertsReached
        );
        // 3. check that issuer respects certification creation period
        ensure!(
            cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number(),
            Error::<T>::IdtyCreationPeriodNotRespected
        );
        Ok(())
    }
}

// implement cert call checks
impl<T: Config> pallet_certification::traits::CheckCertAllowed<IdtyIndex> for Pallet<T> {
    // check the following:
    // - issuer has identity
    // - issuer identity is member
    // - receiver has identity
    // - receiver identity is confirmed and not revoked
    fn check_cert_allowed(issuer: IdtyIndex, receiver: IdtyIndex) -> Result<(), DispatchError> {
        // issuer checks
        // ensure issuer is member
        if let Some(issuer_data) = pallet_identity::Pallet::<T>::identity(issuer) {
            ensure!(
                issuer_data.status == IdtyStatus::Member,
                Error::<T>::IssuerNotMember
            );
        } else {
            return Err(Error::<T>::IdtyNotFound.into());
        }

        // receiver checks
        // ensure receiver identity is confirmed and not revoked
        if let Some(receiver_data) = pallet_identity::Pallet::<T>::identity(receiver) {
            match receiver_data.status {
                // able to receive cert
                IdtyStatus::Unvalidated | IdtyStatus::Member | IdtyStatus::NotMember => {}
                IdtyStatus::Unconfirmed => return Err(Error::<T>::CertToUnconfirmed.into()),
                IdtyStatus::Revoked => return Err(Error::<T>::CertToRevoked.into()),
            };
        } else {
            return Err(Error::<T>::IdtyNotFound.into());
        }
        Ok(())
    }
}

// implement membership call checks
impl<T: Config> sp_membership::traits::CheckMembershipCallAllowed<IdtyIndex> for Pallet<T> {
    // membership claim is only possible when enough certs are received (both wots) and distance is ok
    fn check_idty_allowed_to_claim_membership(idty_index: &IdtyIndex) -> Result<(), DispatchError> {
        let idty_cert_meta = pallet_certification::Pallet::<T>::idty_cert_meta(idty_index);
        ensure!(
            idty_cert_meta.received_count >= T::MinCertForMembership::get(),
            Error::<T>::NotEnoughCertsToClaimMembership
        );
        T::IsDistanceOk::is_distance_ok(idty_index)?;
        Ok(())
    }

    // membership renewal is only possible when identity is member (otherwise it should claim again)
    fn check_idty_allowed_to_renew_membership(idty_index: &IdtyIndex) -> Result<(), DispatchError> {
        if let Some(idty_value) = pallet_identity::Pallet::<T>::identity(idty_index) {
            ensure!(
                idty_value.status == IdtyStatus::Member,
                Error::<T>::IdtyNotAllowedToRenewMembership
            );
            T::IsDistanceOk::is_distance_ok(idty_index)?;
        } else {
            return Err(Error::<T>::IdtyNotFound.into());
        }
        Ok(())
    }
}

// implement membership event handler
impl<T: Config> sp_membership::traits::OnEvent<IdtyIndex> for Pallet<T>
where
    T: pallet_membership::Config,
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex>) {
        match membership_event {
            sp_membership::Event::<IdtyIndex>::MembershipAdded(idty_index) => {
                // when main membership is acquired, tell identity
                // (only used on first membership acquiry)
                pallet_identity::Pallet::<T>::membership_added(*idty_index);
            }
            sp_membership::Event::<IdtyIndex>::MembershipRemoved(idty_index) => {
                // when main membership is lost, tell identity
                pallet_identity::Pallet::<T>::membership_removed(*idty_index);
            }
            sp_membership::Event::<IdtyIndex>::MembershipRenewed(_) => {}
        }
    }
}

// implement identity event handler
impl<T: Config> pallet_identity::traits::OnIdtyChange<T> for Pallet<T> {
    fn on_idty_change(idty_index: IdtyIndex, idty_event: &IdtyEvent<T>) {
        match idty_event {
            // identity just has been created, a cert must be added
            IdtyEvent::Created { creator, .. } => {
                if let Err(e) = <pallet_certification::Pallet<T>>::do_add_cert_checked(
                    *creator, idty_index, true,
                ) {
                    sp_std::if_std! {
                        println!("fail to force add cert: {:?}", e)
                    }
                }
            }
            // TODO split in removed / revoked in two events:
            // if identity is revoked keep it
            // if identity is removed also remove certs
            IdtyEvent::Removed { status } => {
                // try remove membership in any case
                <pallet_membership::Pallet<T>>::do_remove_membership(
                    idty_index,
                    MembershipRemovalReason::Revoked,
                );

                // only remove certs if identity is unvalidated
                match status {
                    IdtyStatus::Unconfirmed | IdtyStatus::Unvalidated => {
                        if let Err(e) =
                            <pallet_certification::Pallet<T>>::remove_all_certs_received_by(
                                frame_system::Origin::<T>::Root.into(),
                                idty_index,
                            )
                        {
                            sp_std::if_std! {
                                println!("fail to remove certs received by some idty: {:?}", e)
                            }
                        }
                    }
                    IdtyStatus::Revoked => {}
                    IdtyStatus::Member | IdtyStatus::NotMember => {
                        sp_std::if_std! {
                            println!("removed non-revoked identity: {:?}", idty_index);
                        }
                    }
                }
            }
        }
    }
}

// implement certification event handlers
// new cert handler
impl<T: Config> pallet_certification::traits::OnNewcert<IdtyIndex> for Pallet<T> {
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    ) {
        if receiver_received_count == T::MinReceivedCertToBeAbleToIssueCert::get() {
            Self::do_apply_first_issuable_on(receiver);
        }
    }
}

// remove cert handler
impl<T: Config> pallet_certification::traits::OnRemovedCert<IdtyIndex> for Pallet<T> {
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        _expiration: bool,
    ) {
        if receiver_received_count < T::MinCertForMembership::get()
            && pallet_membership::Pallet::<T>::is_member(&receiver)
        {
            // expire receiver membership
            <pallet_membership::Pallet<T>>::do_remove_membership(
                receiver,
                MembershipRemovalReason::NotEnoughCerts,
            )
        }
    }
}
