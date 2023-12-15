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
use sp_runtime::traits::IsMember;

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
    pub struct Pallet<T, I = ()>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        frame_system::Config
        + pallet_certification::Config<I, IdtyIndex = IdtyIndex>
        + pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_membership::Config<I, IdtyId = IdtyIndex>
    {
        /// Distance evaluation provider
        type IsDistanceOk: IsDistanceOk<IdtyIndex>;
        #[pallet::constant]
        type FirstIssuableOn: Get<Self::BlockNumber>;
        #[pallet::constant]
        type IsSubWot: Get<bool>;
        #[pallet::constant]
        type MinCertForMembership: Get<u32>;
        #[pallet::constant]
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
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T, I = ()> {
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
impl<AccountId, T: Config<I>, I: 'static> pallet_identity::traits::CheckIdtyCallAllowed<T>
    for Pallet<T, I>
where
    T: frame_system::Config<AccountId = AccountId> + pallet_membership::Config<I>,
{
    // identity creation checks
    fn check_create_identity(creator: IdtyIndex) -> Result<(), DispatchError> {
        // main WoT constraints
        if !T::IsSubWot::get() {
            let cert_meta = pallet_certification::Pallet::<T, I>::idty_cert_meta(creator);
            // perform all checks
            // 1. check that identity has the right to create an identity
            // identity can be member with 5 certifications and still not reach identity creation threshold which could be higher (6, 7...)
            ensure!(
                cert_meta.received_count >= T::MinCertForCreateIdtyRight::get(),
                Error::<T, I>::NotEnoughReceivedCertsToCreateIdty
            );
            // 2. check that issuer can emit one more certification
            // (this is only a partial check)
            ensure!(
                cert_meta.issued_count < T::MaxByIssuer::get(),
                Error::<T, I>::MaxEmittedCertsReached
            );
            // 3. check that issuer respects certification creation period
            ensure!(
                cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number(),
                Error::<T, I>::IdtyCreationPeriodNotRespected
            );
        }
        // TODO (#136) make these trait implementation work on instances rather than static to avoid checking IsSubWot
        // smith subwot can never prevent from creating identity
        Ok(())
    }

    // identity change owner key cheks
    fn change_owner_key(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        // sub WoT prevents from changing identity
        if T::IsSubWot::get() {
            ensure!(
                !pallet_membership::Pallet::<T, I>::is_member(&idty_index),
                Error::<T, I>::NotAllowedToChangeIdtyAddress
            );
        }
        // no constraints for main wot
        Ok(())
    }
}

// implement cert call checks
impl<T: Config<I>, I: 'static> pallet_certification::traits::CheckCertAllowed<IdtyIndex>
    for Pallet<T, I>
// TODO (#136) add the following where clause once checks can be done on pallet instance
// where
//     T: pallet_membership::Config<I>,
{
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
                Error::<T, I>::IssuerNotMember
            );
        } else {
            return Err(Error::<T, I>::IdtyNotFound.into());
        }

        // receiver checks
        // ensure receiver identity is confirmed and not revoked
        if let Some(receiver_data) = pallet_identity::Pallet::<T>::identity(receiver) {
            match receiver_data.status {
                // able to receive cert
                IdtyStatus::Unvalidated | IdtyStatus::Member | IdtyStatus::NotMember => {}
                IdtyStatus::Unconfirmed => return Err(Error::<T, I>::CertToUnconfirmed.into()),
                IdtyStatus::Revoked => return Err(Error::<T, I>::CertToRevoked.into()),
            };
        } else {
            return Err(Error::<T, I>::IdtyNotFound.into());
        }
        Ok(())
    }
}

// implement membership call checks
impl<T: Config<I>, I: 'static> sp_membership::traits::CheckMembershipCallAllowed<IdtyIndex>
    for Pallet<T, I>
{
    // membership claim is only possible when enough certs are received (both wots) and distance is ok
    fn check_idty_allowed_to_claim_membership(idty_index: &IdtyIndex) -> Result<(), DispatchError> {
        let idty_cert_meta = pallet_certification::Pallet::<T, I>::idty_cert_meta(idty_index);
        ensure!(
            idty_cert_meta.received_count >= T::MinCertForMembership::get(),
            Error::<T, I>::NotEnoughCertsToClaimMembership
        );
        T::IsDistanceOk::is_distance_ok(idty_index)?;
        Ok(())
    }

    // membership renewal is only possible when identity is member (otherwise it should claim again)
    fn check_idty_allowed_to_renew_membership(idty_index: &IdtyIndex) -> Result<(), DispatchError> {
        if let Some(idty_value) = pallet_identity::Pallet::<T>::identity(idty_index) {
            ensure!(
                idty_value.status == IdtyStatus::Member,
                Error::<T, I>::IdtyNotAllowedToRenewMembership
            );
            T::IsDistanceOk::is_distance_ok(idty_index)?;
        } else {
            return Err(Error::<T, I>::IdtyNotFound.into());
        }
        Ok(())
    }
}

// implement membership event handler
impl<T: Config<I>, I: 'static> sp_membership::traits::OnEvent<IdtyIndex> for Pallet<T, I>
where
    T: pallet_membership::Config<I>,
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex>) {
        match membership_event {
            sp_membership::Event::<IdtyIndex>::MembershipAdded(idty_index) => {
                if !T::IsSubWot::get() {
                    // when main membership is acquired, tell identity
                    // (only used on first membership acquiry)
                    pallet_identity::Pallet::<T>::membership_added(*idty_index);
                }
            }
            sp_membership::Event::<IdtyIndex>::MembershipRemoved(idty_index) => {
                if !T::IsSubWot::get() {
                    // when main membership is lost, tell identity
                    pallet_identity::Pallet::<T>::membership_removed(*idty_index);
                }
            }
            sp_membership::Event::<IdtyIndex>::MembershipRenewed(_) => {}
        }
    }
}

// implement identity event handler
impl<T: Config<I>, I: 'static> pallet_identity::traits::OnIdtyChange<T> for Pallet<T, I> {
    fn on_idty_change(idty_index: IdtyIndex, idty_event: &IdtyEvent<T>) {
        match idty_event {
            // identity just has been created, a cert must be added
            IdtyEvent::Created { creator, .. } => {
                if let Err(e) = <pallet_certification::Pallet<T, I>>::do_add_cert_checked(
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
                <pallet_membership::Pallet<T, I>>::do_remove_membership(
                    idty_index,
                    MembershipRemovalReason::Revoked,
                );

                // only remove certs if identity is unvalidated
                match status {
                    IdtyStatus::Unconfirmed | IdtyStatus::Unvalidated => {
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
impl<T: Config<I>, I: 'static> pallet_certification::traits::OnNewcert<IdtyIndex> for Pallet<T, I> {
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
impl<T: Config<I>, I: 'static> pallet_certification::traits::OnRemovedCert<IdtyIndex>
    for Pallet<T, I>
{
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u32,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        _expiration: bool,
    ) {
        if receiver_received_count < T::MinCertForMembership::get()
            && pallet_membership::Pallet::<T, I>::is_member(&receiver)
        {
            // expire receiver membership
            <pallet_membership::Pallet<T, I>>::do_remove_membership(
                receiver,
                MembershipRemovalReason::NotEnoughCerts,
            )
        }
    }
}
