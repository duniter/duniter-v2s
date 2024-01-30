// Copyright 2021-2023 Axiom-Team
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod impls;
pub mod traits;
mod types;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use codec::{Codec, Decode, Encode};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::ensure;
use frame_support::pallet_prelude::Get;
use frame_support::pallet_prelude::RuntimeDebug;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::OriginFor;
use scale_info::TypeInfo;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::traits::IsMember;
use sp_std::fmt::Debug;
use sp_std::prelude::*;

use crate::traits::OnSmithDelete;
pub use crate::weights::WeightInfo;
pub use pallet::*;
use pallet_authority_members::SessionIndex;
pub use types::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SmithRemovalReason {
    LostMembership,
    OfflineTooLong,
    Blacklisted,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SmithStatus {
    /// The identity has been invited by a smith but has not accepted yet
    Invited,
    /// The identity has accepted to eventually become a smith
    Pending,
    /// The identity has reached the requirements to become a smith and can now goGoOnline() or invite/certify other smiths
    Smith,
    /// The identity has been removed from the smiths set but is kept to keep track of its certifications
    Excluded,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::StorageVersion;
    use pallet_authority_members::SessionIndex;
    use sp_runtime::traits::{Convert, IsMember};
    use sp_std::collections::btree_map::BTreeMap;
    use sp_std::vec;
    use sp_std::vec::Vec;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// The pallet's config trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// To only allow WoT members to be invited
        type IsWoTMember: IsMember<Self::IdtyIndex>;
        /// Notify when a smith is removed (for authority-members to react)
        type OnSmithDelete: traits::OnSmithDelete<Self::IdtyIndex>;
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A short identity index.
        type IdtyIndex: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen;
        /// Identifier for an authority-member
        type MemberId: Copy + Ord + MaybeSerializeDeserialize + Parameter;
        /// Something that gives the IdtyId of an AccountId
        type IdtyIdOf: Convert<Self::AccountId, Option<Self::IdtyIndex>>;
        /// Something that give the owner key of an identity
        type OwnerKeyOf: Convert<Self::IdtyIndex, Option<Self::AccountId>>;
        /// Something that gives the IdtyId of an AccountId
        type IdtyIdOfAuthorityId: Convert<Self::MemberId, Option<Self::IdtyIndex>>;
        /// Maximum number of active certifications by issuer
        #[pallet::constant]
        type MaxByIssuer: Get<u32>;
        /// Minimum number of certifications to become a Smith
        #[pallet::constant]
        type MinCertForMembership: Get<u32>;
        /// Maximum duration of inactivity before a smith is removed
        #[pallet::constant]
        type SmithInactivityMaxDuration: Get<u32>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
    }

    /// Events type.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An identity is being inivited to become a smith.
        InvitationSent {
            receiver: T::IdtyIndex,
            issuer: T::IdtyIndex,
        },
        /// The invitation has been accepted.
        InvitationAccepted { idty_index: T::IdtyIndex },
        /// Certification received
        SmithCertAdded {
            receiver: T::IdtyIndex,
            issuer: T::IdtyIndex,
        },
        /// Certification lost
        SmithCertRemoved {
            receiver: T::IdtyIndex,
            issuer: T::IdtyIndex,
        },
        /// A smith gathered enough certifications to become an authority (can call `go_online()`).
        SmithMembershipAdded { idty_index: T::IdtyIndex },
        /// A smith has been removed from the smiths set.
        SmithMembershipRemoved { idty_index: T::IdtyIndex },
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_smiths: BTreeMap<T::IdtyIndex, (bool, Vec<T::IdtyIndex>)>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_smiths: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            CurrentSession::<T>::put(0);
            let mut cert_meta_by_issuer = BTreeMap::<T::IdtyIndex, Vec<T::IdtyIndex>>::new();
            for (receiver, (is_online, issuers)) in &self.initial_smiths {
                // Forbid self-cert
                assert!(
                    !issuers.contains(receiver),
                    "Identity cannot certify it-self."
                );

                let mut issuers_: Vec<_> = Vec::with_capacity(issuers.len());
                for issuer in issuers {
                    // Count issued certs
                    cert_meta_by_issuer
                        .entry(*issuer)
                        .or_insert(vec![])
                        .push(*receiver);
                    issuers_.push(*issuer);
                }

                // Write CertsByReceiver
                issuers_.sort();
                let issuers_count = issuers_.len();
                let smith_status = if issuers_count >= T::MinCertForMembership::get() as usize {
                    SmithStatus::Smith
                } else {
                    SmithStatus::Pending
                };
                Smiths::<T>::insert(
                    receiver,
                    SmithMeta {
                        status: smith_status,
                        expires_on: if *is_online {
                            None
                        } else {
                            Some(CurrentSession::<T>::get() + T::SmithInactivityMaxDuration::get())
                        },
                        issued_certs: vec![],
                        received_certs: issuers_,
                    },
                );
                ExpiresOn::<T>::append(
                    CurrentSession::<T>::get() + T::SmithInactivityMaxDuration::get(),
                    receiver,
                );
            }

            for (issuer, issued_certs) in cert_meta_by_issuer {
                // Write CertsByIssuer
                Smiths::<T>::mutate(issuer, |maybe_smith_meta| {
                    if let Some(smith_meta) = maybe_smith_meta {
                        smith_meta.issued_certs = issued_certs;
                    }
                });
            }
        }
    }

    /// maps identity index to smith status
    #[pallet::storage]
    #[pallet::getter(fn smiths)]
    pub type Smiths<T: Config> =
        StorageMap<_, Twox64Concat, T::IdtyIndex, SmithMeta<T::IdtyIndex>, OptionQuery>;

    /// maps session index to possible smith removals
    #[pallet::storage]
    #[pallet::getter(fn expires_on)]
    pub type ExpiresOn<T: Config> =
        StorageMap<_, Twox64Concat, SessionIndex, Vec<T::IdtyIndex>, OptionQuery>;

    /// stores the current session index
    #[pallet::storage]
    #[pallet::getter(fn current_session)]
    pub type CurrentSession<T: Config> = StorageValue<_, SessionIndex, ValueQuery>;

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Issuer of anything (invitation, acceptance, certification) must have an identity ID
        OriginMustHaveAnIdentity,
        /// Issuer must be known as a potential smith
        OriginHasNeverBeenInvited,
        /// Invitation is reseverd to smiths
        InvitationIsASmithPrivilege,
        /// Invitation is reseverd to online smiths
        InvitationIsAOnlineSmithPrivilege,
        /// Invitation must not have been accepted yet
        InvitationAlreadyAccepted,
        /// Invitation of an already known smith is forbidden except if it has been excluded
        InvitationOfExistingNonExcluded,
        /// Invitation of a non-member (of the WoT) is forbidden
        InvitationOfNonMember,
        /// Certification cannot be made on someone who has not accepted an invitation
        CertificationMustBeAgreed,
        /// Certification cannot be made on excluded
        CertificationOnExcludedIsForbidden,
        /// Issuer must be a smith
        CertificationIsASmithPrivilege,
        /// Only online smiths can certify
        CertificationIsAOnlineSmithPrivilege,
        /// Smith cannot certify itself
        CertificationOfSelfIsForbidden,
        /// Receiver must be invited by another smith
        CertificationReceiverMustHaveBeenInvited,
        /// Receiver must not already have this certification
        CertificationAlreadyExists,
        /// A smith has a limited stock of certifications
        CertificationStockFullyConsumed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Invite a WoT member to try becoming a Smith
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::invite_smith())]
        pub fn invite_smith(
            origin: OriginFor<T>,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            let issuer =
                T::IdtyIdOf::convert(who.clone()).ok_or(Error::<T>::OriginMustHaveAnIdentity)?;
            Self::check_invite_smith(issuer, receiver)?;
            Self::do_invite_smith(issuer, receiver);
            Ok(().into())
        }

        /// Accept an invitation (must have been invited first)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::accept_invitation())]
        pub fn accept_invitation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            let receiver =
                T::IdtyIdOf::convert(who.clone()).ok_or(Error::<T>::OriginMustHaveAnIdentity)?;
            Self::check_accept_invitation(receiver)?;
            Self::do_accept_invitation(receiver)?;
            Ok(().into())
        }

        /// Certify an invited smith which can lead the certified to become a Smith
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::certify_smith())]
        pub fn certify_smith(
            origin: OriginFor<T>,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let issuer =
                T::IdtyIdOf::convert(who.clone()).ok_or(Error::<T>::OriginMustHaveAnIdentity)?;
            Self::check_certify_smith(issuer, receiver)?;
            Self::do_certify_smith(receiver, issuer);
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn check_invite_smith(
        issuer: T::IdtyIndex,
        receiver: T::IdtyIndex,
    ) -> DispatchResultWithPostInfo {
        let issuer = Smiths::<T>::get(issuer).ok_or(Error::<T>::OriginHasNeverBeenInvited)?;
        ensure!(
            issuer.status == SmithStatus::Smith,
            Error::<T>::InvitationIsASmithPrivilege
        );
        ensure!(
            issuer.expires_on.is_none(),
            Error::<T>::InvitationIsAOnlineSmithPrivilege
        );
        if let Some(receiver_meta) = Smiths::<T>::get(receiver) {
            ensure!(
                receiver_meta.status == SmithStatus::Excluded,
                Error::<T>::InvitationOfExistingNonExcluded
            );
        }
        ensure!(
            T::IsWoTMember::is_member(&receiver),
            Error::<T>::InvitationOfNonMember
        );

        Ok(().into())
    }

    fn do_invite_smith(issuer: T::IdtyIndex, receiver: T::IdtyIndex) {
        let new_expires_on = CurrentSession::<T>::get() + T::SmithInactivityMaxDuration::get();
        let mut existing = Smiths::<T>::get(receiver).unwrap_or_default();
        existing.status = SmithStatus::Invited;
        existing.expires_on = Some(new_expires_on);
        existing.received_certs = vec![];
        Smiths::<T>::insert(receiver, existing);
        ExpiresOn::<T>::append(new_expires_on, receiver);
        Self::deposit_event(Event::<T>::InvitationSent { receiver, issuer });
    }

    fn check_accept_invitation(receiver: T::IdtyIndex) -> DispatchResultWithPostInfo {
        let pretender_status = Smiths::<T>::get(receiver)
            .ok_or(Error::<T>::OriginHasNeverBeenInvited)?
            .status;
        ensure!(
            pretender_status == SmithStatus::Invited,
            Error::<T>::InvitationAlreadyAccepted
        );
        Ok(().into())
    }

    fn do_accept_invitation(receiver: T::IdtyIndex) -> DispatchResultWithPostInfo {
        Smiths::<T>::mutate(receiver, |maybe_smith_meta| {
            if let Some(smith_meta) = maybe_smith_meta {
                smith_meta.status = SmithStatus::Pending;
            }
        });
        Self::deposit_event(Event::<T>::InvitationAccepted {
            idty_index: receiver,
        });
        Ok(().into())
    }

    fn check_certify_smith(
        issuer_index: T::IdtyIndex,
        receiver_index: T::IdtyIndex,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            issuer_index != receiver_index,
            Error::<T>::CertificationOfSelfIsForbidden
        );
        let issuer = Smiths::<T>::get(issuer_index).ok_or(Error::<T>::OriginHasNeverBeenInvited)?;
        ensure!(
            issuer.status == SmithStatus::Smith,
            Error::<T>::CertificationIsASmithPrivilege
        );
        ensure!(
            issuer.expires_on.is_none(),
            Error::<T>::CertificationIsAOnlineSmithPrivilege
        );
        let issued_certs = issuer.issued_certs.len();
        ensure!(
            issued_certs < T::MaxByIssuer::get() as usize,
            Error::<T>::CertificationStockFullyConsumed
        );
        let receiver = Smiths::<T>::get(receiver_index)
            .ok_or(Error::<T>::CertificationReceiverMustHaveBeenInvited)?;
        ensure!(
            receiver.status != SmithStatus::Invited,
            Error::<T>::CertificationMustBeAgreed
        );
        ensure!(
            receiver.status != SmithStatus::Excluded,
            Error::<T>::CertificationOnExcludedIsForbidden
        );
        ensure!(
            receiver
                .received_certs
                .binary_search(&issuer_index)
                .is_err(),
            Error::<T>::CertificationAlreadyExists
        );

        Ok(().into())
    }

    fn do_certify_smith(receiver: T::IdtyIndex, issuer: T::IdtyIndex) {
        // - adds a certification in issuer issued list
        Smiths::<T>::mutate(issuer, |maybe_smith_meta| {
            if let Some(smith_meta) = maybe_smith_meta {
                smith_meta.issued_certs.push(receiver);
                smith_meta.issued_certs.sort();
            }
        });
        Smiths::<T>::mutate(receiver, |maybe_smith_meta| {
            if let Some(smith_meta) = maybe_smith_meta {
                // - adds a certification in receiver received list
                smith_meta.received_certs.push(issuer);
                smith_meta.received_certs.sort();
                Self::deposit_event(Event::<T>::SmithCertAdded { receiver, issuer });

                // - receiving a certification either lead us to Pending or Smith status
                let previous_status = smith_meta.status;
                smith_meta.status =
                    if smith_meta.received_certs.len() >= T::MinCertForMembership::get() as usize {
                        // - if the number of certification received by the receiver is enough, win the Smith status (or keep it)
                        SmithStatus::Smith
                    } else {
                        // - otherwise we are (still) a pending smith
                        SmithStatus::Pending
                    };

                if previous_status != SmithStatus::Smith {
                    // - postpone the expiration: a Pending smith cannot do anything but wait
                    // this postponement is here to ease the process of becoming a smith
                    let new_expires_on =
                        CurrentSession::<T>::get() + T::SmithInactivityMaxDuration::get();
                    smith_meta.expires_on = Some(new_expires_on);
                    ExpiresOn::<T>::append(new_expires_on, receiver);
                }

                // - if the status is smith but wasn't, notify that smith gained membership
                if smith_meta.status == SmithStatus::Smith && previous_status != SmithStatus::Smith
                {
                    Self::deposit_event(Event::<T>::SmithMembershipAdded {
                        idty_index: receiver,
                    });
                }
                // TODO: (optimization) unschedule old expiry
            }
        });
    }

    fn on_exclude_expired_smiths(at: SessionIndex) {
        if let Some(smiths_to_remove) = ExpiresOn::<T>::get(at) {
            for smith in smiths_to_remove {
                if let Some(smith_meta) = Smiths::<T>::get(smith) {
                    if let Some(expires_on) = smith_meta.expires_on {
                        if expires_on == at {
                            Self::_do_exclude_smith(smith, SmithRemovalReason::OfflineTooLong);
                        }
                    }
                }
            }
        }
    }

    pub fn on_removed_wot_member(idty_index: T::IdtyIndex) {
        if Smiths::<T>::get(idty_index).is_some() {
            Self::_do_exclude_smith(idty_index, SmithRemovalReason::LostMembership);
        }
    }

    fn _do_exclude_smith(receiver: T::IdtyIndex, reason: SmithRemovalReason) {
        let mut lost_certs = vec![];
        Smiths::<T>::mutate(receiver, |maybe_smith_meta| {
            if let Some(smith_meta) = maybe_smith_meta {
                smith_meta.expires_on = None;
                smith_meta.status = SmithStatus::Excluded;
                for cert in &smith_meta.received_certs {
                    lost_certs.push(*cert);
                }
                smith_meta.received_certs = vec![];
                // N.B.: the issued certs are kept in case the smith joins back
            }
        });
        // We remove the lost certs from their issuer's stock
        for lost_cert_issuer in lost_certs {
            Smiths::<T>::mutate(lost_cert_issuer, |maybe_smith_meta| {
                if let Some(smith_meta) = maybe_smith_meta {
                    if let Ok(index) = smith_meta.issued_certs.binary_search(&receiver) {
                        smith_meta.issued_certs.remove(index);
                        Self::deposit_event(Event::<T>::SmithCertRemoved {
                            receiver,
                            issuer: lost_cert_issuer,
                        });
                    }
                }
            });
        }
        // Deletion done: notify (authority-members) for cascading
        T::OnSmithDelete::on_smith_delete(receiver, reason);
        Self::deposit_event(Event::<T>::SmithMembershipRemoved {
            idty_index: receiver,
        });
    }

    pub fn on_smith_goes_online(idty_index: T::IdtyIndex) {
        if let Some(smith_meta) = Smiths::<T>::get(idty_index) {
            if smith_meta.expires_on.is_some() {
                Smiths::<T>::mutate(idty_index, |maybe_smith_meta| {
                    if let Some(smith_meta) = maybe_smith_meta {
                        // As long as the smith is online, it cannot expire
                        smith_meta.expires_on = None;
                        // FIXME: unschedule old expiry
                    }
                });
            }
        }
    }

    pub fn on_smith_goes_offline(idty_index: T::IdtyIndex) {
        if let Some(smith_meta) = Smiths::<T>::get(idty_index) {
            if smith_meta.expires_on.is_none() {
                Smiths::<T>::mutate(idty_index, |maybe_smith_meta| {
                    if let Some(smith_meta) = maybe_smith_meta {
                        // As long as the smith is online, it cannot expire
                        let new_expires_on =
                            CurrentSession::<T>::get() + T::SmithInactivityMaxDuration::get();
                        smith_meta.expires_on = Some(new_expires_on);
                        ExpiresOn::<T>::append(new_expires_on, idty_index);
                    }
                });
            }
        }
    }

    fn provide_is_member(idty_id: &T::IdtyIndex) -> bool {
        let Some(smith) = Smiths::<T>::get(idty_id) else {
            return false;
        };
        smith.status == SmithStatus::Smith
    }
}

impl<T: Config> sp_runtime::traits::IsMember<T::IdtyIndex> for Pallet<T> {
    fn is_member(idty_id: &T::IdtyIndex) -> bool {
        Self::provide_is_member(idty_id)
    }
}
