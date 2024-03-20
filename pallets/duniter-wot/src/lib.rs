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

use frame_support::pallet_prelude::*;
use pallet_certification::traits::SetNextIssuableOn;
use pallet_identity::IdtyStatus;
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
        #[pallet::constant]
        type FirstIssuableOn: Get<frame_system::pallet_prelude::BlockNumberFor<Self>>;
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
        /// Insufficient certifications received.
        NotEnoughCerts,
        /// Target status is incompatible with this operation.
        // - Membership can not be added/renewed with this status
        // - Certification can not be added to identity with this status
        TargetStatusInvalid,
        /// Identity creation period not respected.
        IdtyCreationPeriodNotRespected,
        /// Insufficient received certifications to create identity.
        NotEnoughReceivedCertsToCreateIdty,
        /// Maximum number of emitted certifications reached.
        MaxEmittedCertsReached,
        /// Issuer cannot emit a certification because it is not member.
        IssuerNotMember,
        /// Issuer or receiver not found.
        IdtyNotFound,
        /// Membership can only be renewed after an antispam delay.
        MembershipRenewalPeriodNotRespected,
    }
}

/// Implementing identity call allowance check for the pallet.
impl<AccountId, T: Config> pallet_identity::traits::CheckIdtyCallAllowed<T> for Pallet<T>
where
    T: frame_system::Config<AccountId = AccountId> + pallet_membership::Config,
{
    /// Checks if identity creation is allowed.
    /// This implementation checks the following:
    ///
    /// - Whether the identity has the right to create an identity.
    /// - Whether the issuer can emit a certification.
    /// - Whether the issuer respect creation period.
    fn check_create_identity(creator: IdtyIndex) -> Result<(), DispatchError> {
        let cert_meta = pallet_certification::Pallet::<T>::idty_cert_meta(creator);

        // 1. Check that the identity has the right to create an identity
        // Identity can be a member with 5 certifications and still not reach the identity creation threshold, which could be higher (6, 7...)
        ensure!(
            cert_meta.received_count >= T::MinCertForCreateIdtyRight::get(),
            Error::<T>::NotEnoughReceivedCertsToCreateIdty
        );

        // 2. Check that the issuer can emit one more certification (partial check)
        ensure!(
            cert_meta.issued_count < T::MaxByIssuer::get(),
            Error::<T>::MaxEmittedCertsReached
        );

        // 3. Check that the issuer respects certification creation period
        ensure!(
            cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number(),
            Error::<T>::IdtyCreationPeriodNotRespected
        );
        Ok(())
    }
}

/// Implementing certification allowance check for the pallet.
impl<T: Config> pallet_certification::traits::CheckCertAllowed<IdtyIndex> for Pallet<T> {
    /// Checks if certification is allowed.
    /// This implementation checks the following:
    ///
    /// - Whether the issuer has an identity.
    /// - Whether the issuer's identity is a member.
    /// - Whether the receiver has an identity.
    /// - Whether the receiver's identity is confirmed and not revoked.
    fn check_cert_allowed(issuer: IdtyIndex, receiver: IdtyIndex) -> Result<(), DispatchError> {
        // Issuer checks
        // Ensure issuer is a member
        let issuer_data =
            pallet_identity::Pallet::<T>::identity(issuer).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            issuer_data.status == IdtyStatus::Member,
            Error::<T>::IssuerNotMember
        );

        // Receiver checks
        // Ensure receiver identity is confirmed and not revoked
        let receiver_data =
            pallet_identity::Pallet::<T>::identity(receiver).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            receiver_data.status == IdtyStatus::Unvalidated
                || receiver_data.status == IdtyStatus::Member
                || receiver_data.status == IdtyStatus::NotMember,
            Error::<T>::TargetStatusInvalid
        );
        Ok(())
    }
}

/// Implementing membership operation checks for the pallet.
impl<T: Config> sp_membership::traits::CheckMembershipOpAllowed<IdtyIndex> for Pallet<T> {
    /// This implementation checks the following:
    ///
    /// - Whether the identity's status is unvalidated or not a member.
    /// - The count of certifications associated with the identity.
    fn check_add_membership(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        // Check identity status
        let idty_value =
            pallet_identity::Pallet::<T>::identity(idty_index).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            idty_value.status == IdtyStatus::Unvalidated
                || idty_value.status == IdtyStatus::NotMember,
            Error::<T>::TargetStatusInvalid
        );

        // Check certificate count
        check_cert_count::<T>(idty_index)?;
        Ok(())
    }

    /// This implementation checks the following:
    ///
    /// - Whether the identity's status is member.
    ///
    /// Note: There is no need to check certification count since losing certifications makes membership expire.
    /// Membership renewal is only possible when identity is member.
    fn check_renew_membership(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        let idty_value =
            pallet_identity::Pallet::<T>::identity(idty_index).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            idty_value.status == IdtyStatus::Member,
            Error::<T>::TargetStatusInvalid
        );
        Ok(())
    }
}

/// Implementing membership event handling for the pallet.
impl<T: Config> sp_membership::traits::OnNewMembership<IdtyIndex> for Pallet<T>
where
    T: pallet_membership::Config,
{
    /// This implementation notifies the identity pallet when a main membership is acquired.
    /// It is only used on the first membership acquisition.
    fn on_created(idty_index: &IdtyIndex) {
        pallet_identity::Pallet::<T>::membership_added(*idty_index);
    }

    fn on_renewed(_idty_index: &IdtyIndex) {}
}

/// Implementing membership removal event handling for the pallet.
impl<T: Config> sp_membership::traits::OnRemoveMembership<IdtyIndex> for Pallet<T>
where
    T: pallet_membership::Config,
{
    /// This implementation notifies the identity pallet when a main membership is lost.
    fn on_removed(idty_index: &IdtyIndex) -> Weight {
        pallet_identity::Pallet::<T>::membership_removed(*idty_index)
    }
}

/// Implementing the identity event handler for the pallet.
impl<T: Config> pallet_identity::traits::OnNewIdty<T> for Pallet<T> {
    /// This implementation adds a certificate when a new identity is created.
    fn on_created(idty_index: &IdtyIndex, creator: &IdtyIndex) {
        if let Err(e) =
            <pallet_certification::Pallet<T>>::do_add_cert_checked(*creator, *idty_index, true)
        {
            sp_std::if_std! {
                println!("fail to force add cert: {:?}", e)
            }
        }
    }
}

/// Implementing identity removal event handling for the pallet.
impl<T: Config> pallet_identity::traits::OnRemoveIdty<T> for Pallet<T> {
    /// This implementation removes both membership and certificates associated with the identity.
    fn on_removed(idty_index: &IdtyIndex) -> Weight {
        let mut weight = Self::on_revoked(idty_index);
        weight = weight.saturating_add(
            <pallet_certification::Pallet<T>>::do_remove_all_certs_received_by(*idty_index),
        );
        weight
    }

    /// This implementation removes membership only.
    fn on_revoked(idty_index: &IdtyIndex) -> Weight {
        let mut weight = Weight::zero();
        weight = weight.saturating_add(<pallet_membership::Pallet<T>>::do_remove_membership(
            *idty_index,
            MembershipRemovalReason::Revoked,
        ));
        weight
    }
}

/// Implementing the certification event handler for the pallet.
impl<T: Config> pallet_certification::traits::OnNewcert<IdtyIndex> for Pallet<T> {
    /// This implementation checks if the receiver has received enough certificates to be able to issue certificates,
    /// and applies the first issuable if the condition is met.
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

/// Implementing the certification removal event handler for the pallet.
impl<T: Config> pallet_certification::traits::OnRemovedCert<IdtyIndex> for Pallet<T> {
    /// This implementation checks if the receiver has received fewer certificates than required for membership,
    /// and if so, and the receiver is a member, it expires the receiver's membership.
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
            // Expire receiver membership
            <pallet_membership::Pallet<T>>::do_remove_membership(
                receiver,
                MembershipRemovalReason::NotEnoughCerts,
            );
        }
    }
}

/// Implementing the valid distance status event handler for the pallet.
impl<T: Config + pallet_distance::Config> pallet_distance::traits::OnValidDistanceStatus<T>
    for Pallet<T>
{
    /// This implementation handles different scenarios based on the identity's status:
    ///
    /// - For `Unconfirmed` or `Revoked` identities, no action is taken.
    /// - For `Unvalidated` or `NotMember` identities, an attempt is made to add membership.
    /// - For `Member` identities, an attempt is made to renew membership.
    fn on_valid_distance_status(idty_index: IdtyIndex) {
        if let Some(identity) = pallet_identity::Identities::<T>::get(idty_index) {
            match identity.status {
                IdtyStatus::Unconfirmed | IdtyStatus::Revoked => {
                    // IdtyStatus::Unconfirmed
                    // distance evaluation request should never happen for unconfirmed identity
                    // IdtyStatus::Revoked
                    // the identity can have been revoked during distance evaluation by the oracle
                }

                IdtyStatus::Unvalidated | IdtyStatus::NotMember => {
                    // IdtyStatus::Unvalidated
                    // normal scenario for first entry
                    // IdtyStatus::NotMember
                    // normal scenario for re-entry
                    // the following can fail if a certification expired during distance evaluation
                    // otherwise it should succeed
                    let _ = pallet_membership::Pallet::<T>::try_add_membership(idty_index);
                    // sp_std::if_std! {
                    //     if let Err(e) = r {
                    //         print!("failed to claim identity when distance status was found ok: ");
                    //         println!("{:?}", idty_index);
                    //         println!("reason: {:?}", e);
                    //     }
                    // }
                }
                IdtyStatus::Member => {
                    // IdtyStatus::Member
                    // normal scenario for renewal
                    // should succeed
                    let _ = pallet_membership::Pallet::<T>::try_renew_membership(idty_index);
                    // sp_std::if_std! {
                    //     if let Err(e) = r {
                    //         print!("failed to renew identity when distance status was found ok: ");
                    //         println!("{:?}", idty_index);
                    //         println!("reason: {:?}", e);
                    //     }
                    // }
                }
            }
        } else {
            // identity was removed before distance status was found
            // so it's ok to do nothing
            sp_std::if_std! {
                println!("identity was removed before distance status was found: {:?}", idty_index);
            }
        }
    }
}

/// Implementing the request distance evaluation check for the pallet.
impl<T: Config + pallet_distance::Config> pallet_distance::traits::CheckRequestDistanceEvaluation<T>
    for Pallet<T>
{
    /// This implementation performs the following checks:
    ///
    /// - Membership renewal anti-spam check: Ensures that membership renewal requests respect the anti-spam period.
    /// - Certificate count check: Ensures that the identity has a sufficient number of certificates.
    fn check_request_distance_evaluation(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        // Check membership renewal anti-spam
        let maybe_membership_data = pallet_membership::Pallet::<T>::membership(idty_index);
        if let Some(membership_data) = maybe_membership_data {
            // If membership data exists, this is for a renewal, apply anti-spam
            ensure!(
                // current_block > expiration block - membership period + renewal period
                membership_data.expire_on
                    + <T as pallet_membership::Config>::MembershipRenewalPeriod::get()
                    < frame_system::Pallet::<T>::block_number()
                        + <T as pallet_membership::Config>::MembershipPeriod::get(),
                Error::<T>::MembershipRenewalPeriodNotRespected
            );
        };
        // Check certificate count
        check_cert_count::<T>(idty_index)?;
        Ok(())
    }
}

/// Checks the certificate count for an identity.
fn check_cert_count<T: Config>(idty_index: IdtyIndex) -> Result<(), DispatchError> {
    let idty_cert_meta = pallet_certification::Pallet::<T>::idty_cert_meta(idty_index);
    ensure!(
        idty_cert_meta.received_count >= T::MinCertForMembership::get(),
        Error::<T>::NotEnoughCerts
    );
    Ok(())
}
