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
        let issuer_data =
            pallet_identity::Pallet::<T>::identity(issuer).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            issuer_data.status == IdtyStatus::Member,
            Error::<T>::IssuerNotMember
        );

        // receiver checks
        // ensure receiver identity is confirmed and not revoked
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

// implement membership call checks
impl<T: Config> sp_membership::traits::CheckMembershipOpAllowed<IdtyIndex> for Pallet<T> {
    fn check_add_membership(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        // check identity status
        let idty_value =
            pallet_identity::Pallet::<T>::identity(idty_index).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            idty_value.status == IdtyStatus::Unvalidated
                || idty_value.status == IdtyStatus::NotMember,
            Error::<T>::TargetStatusInvalid
        );
        // check cert count
        check_cert_count::<T>(idty_index)?;
        Ok(())
    }

    // membership renewal is only possible when identity is member (otherwise it should claim again)
    fn check_renew_membership(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        // check identity status
        let idty_value =
            pallet_identity::Pallet::<T>::identity(idty_index).ok_or(Error::<T>::IdtyNotFound)?;
        ensure!(
            idty_value.status == IdtyStatus::Member,
            Error::<T>::TargetStatusInvalid
        );
        // no need to check certification count since loosing certifications make membership expire
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
            // we could split this event in removed / revoked:
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

/// valid distance status handler
impl<T: Config + pallet_distance::Config> pallet_distance::traits::OnValidDistanceStatus<T>
    for Pallet<T>
{
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

/// distance evaluation request allowed check
impl<T: Config + pallet_distance::Config> pallet_distance::traits::CheckRequestDistanceEvaluation<T>
    for Pallet<T>
{
    fn check_request_distance_evaluation(idty_index: IdtyIndex) -> Result<(), DispatchError> {
        // check membership renewal antispam
        let maybe_membership_data = pallet_membership::Pallet::<T>::membership(idty_index);
        if let Some(membership_data) = maybe_membership_data {
            // if membership data exists, this is for a renewal, apply antispam
            ensure!(
                // current_block > expiration block - membership period + renewal period
                membership_data.expire_on
                    + <T as pallet_membership::Config>::MembershipRenewalPeriod::get()
                    < frame_system::Pallet::<T>::block_number()
                        + <T as pallet_membership::Config>::MembershipPeriod::get(),
                Error::<T>::MembershipRenewalPeriodNotRespected
            );
        };
        // check cert count
        check_cert_count::<T>(idty_index)?;
        Ok(())
    }
}

/// check certification count
fn check_cert_count<T: Config>(idty_index: IdtyIndex) -> Result<(), DispatchError> {
    let idty_cert_meta = pallet_certification::Pallet::<T>::idty_cert_meta(idty_index);
    ensure!(
        idty_cert_meta.received_count >= T::MinCertForMembership::get(),
        Error::<T>::NotEnoughCerts
    );
    Ok(())
}
