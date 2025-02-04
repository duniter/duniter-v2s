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

//! # Duniter Certification Pallet
//!
//! This pallet manages certification creation and deletion.
//!
//! Duniter certifications are the *edges* in the Duniter [Web of Trust](../duniter-wot/). They can have different meanings:
//!
//! - In the case of the main WoT, they mean "I have met this person in real life and trust them" (see Ğ1 Licence).
//! - In the case of the smith sub-WoT, they mean "I trust this person to be able to run Duniter securely" (see smith Licence).

#![cfg_attr(not(feature = "std"), no_std)]

pub mod benchmarking;

pub mod traits;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use crate::traits::*;
use codec::Codec;
use duniter_primitives::Idty;
use frame_support::{pallet_prelude::*, traits::StorageVersion};
use scale_info::prelude::{collections::BTreeMap, fmt::Debug, vec::Vec};
use sp_runtime::traits::AtLeast32BitUnsigned;

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The minimum duration (in blocks) between two certifications issued by the same issuer.
        #[pallet::constant]
        type CertPeriod: Get<BlockNumberFor<Self>>;

        /// A short identity index type.
        type IdtyIndex: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen;

        /// A type that provides methods to get the IdtyIndex of an AccountId and vice versa.
        type IdtyAttr: duniter_primitives::Idty<Self::IdtyIndex, Self::AccountId>;

        /// A type that provides a method to check if issuing a certification is allowed.
        type CheckCertAllowed: CheckCertAllowed<Self::IdtyIndex>;

        /// The maximum number of active certifications that can be issued by a single issuer.
        #[pallet::constant]
        type MaxByIssuer: Get<u32>;

        /// The minimum number of certifications received that an identity must have
        /// to be allowed to issue a certification.
        #[pallet::constant]
        type MinReceivedCertToBeAbleToIssueCert: Get<u32>;

        /// A handler that is called when a new certification event (`NewCert`) occurs.
        type OnNewcert: OnNewcert<Self::IdtyIndex>;

        /// A handler that is called when a certification is removed (`RemovedCert`).
        type OnRemovedCert: OnRemovedCert<Self::IdtyIndex>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;

        /// The duration (in blocks) for which a certification remains valid.
        #[pallet::constant]
        type ValidityPeriod: Get<BlockNumberFor<Self>>;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    #[allow(clippy::type_complexity)]
    pub struct GenesisConfig<T: Config> {
        pub apply_cert_period_at_genesis: bool,
        pub certs_by_receiver:
            BTreeMap<T::IdtyIndex, BTreeMap<T::IdtyIndex, Option<BlockNumberFor<T>>>>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                apply_cert_period_at_genesis: false,
                certs_by_receiver: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let mut cert_meta_by_issuer =
                BTreeMap::<T::IdtyIndex, IdtyCertMeta<BlockNumberFor<T>>>::new();
            let mut certs_removable_on =
                BTreeMap::<BlockNumberFor<T>, Vec<(T::IdtyIndex, T::IdtyIndex)>>::new();
            for (receiver, issuers) in &self.certs_by_receiver {
                // Forbid self-cert
                assert!(
                    !issuers.contains_key(receiver),
                    "Identity cannot certify it-self."
                );

                // We should insert cert_meta for receivers that have not issued any cert.
                cert_meta_by_issuer
                    .entry(*receiver)
                    .or_insert(IdtyCertMeta {
                        issued_count: 0,
                        next_issuable_on: sp_runtime::traits::Zero::zero(),
                        received_count: 0,
                    })
                    .received_count = issuers.len() as u32;

                let mut issuers_: Vec<_> = Vec::with_capacity(issuers.len());
                for (issuer, maybe_removable_on) in issuers {
                    // Count issued certs
                    cert_meta_by_issuer
                        .entry(*issuer)
                        .or_insert(IdtyCertMeta {
                            issued_count: 0,
                            next_issuable_on: sp_runtime::traits::Zero::zero(),
                            received_count: 0,
                        })
                        .issued_count += 1;

                    // Compute and store removable_on
                    let removable_on = maybe_removable_on.unwrap_or_else(T::ValidityPeriod::get);
                    issuers_.push((*issuer, removable_on));

                    // Prepare CertsRemovableOn
                    certs_removable_on
                        .entry(removable_on)
                        .or_default()
                        .push((*issuer, *receiver));

                    if self.apply_cert_period_at_genesis {
                        let issuer_next_issuable_on = removable_on
                            .saturating_add(T::CertPeriod::get())
                            .saturating_sub(T::ValidityPeriod::get());
                        if let Some(cert_meta) = cert_meta_by_issuer.get_mut(issuer) {
                            if cert_meta.next_issuable_on < issuer_next_issuable_on {
                                cert_meta.next_issuable_on = issuer_next_issuable_on;
                            }
                        }
                    }
                }

                // Write CertsByReceiver
                issuers_.sort();
                CertsByReceiver::<T>::insert(receiver, issuers_);
            }

            // Write StorageIdtyCertMeta
            for (issuer, cert_meta) in cert_meta_by_issuer {
                assert!(
                    !cert_meta.issued_count >= T::MaxByIssuer::get(),
                    "Identity n°{:?} exceed MaxByIssuer.",
                    issuer
                );
                assert!(
                    !cert_meta.received_count >= T::MinReceivedCertToBeAbleToIssueCert::get(),
                    "Identity n°{:?} not respect MinReceivedCertToBeAbleToIssueCert.",
                    issuer
                );
                StorageIdtyCertMeta::<T>::insert(issuer, cert_meta);
            }
            // Write storage CertsRemovableOn
            for (removable_on, certs) in certs_removable_on {
                CertsRemovableOn::<T>::insert(removable_on, certs);
            }
        }
    }

    // STORAGE //

    /// The certification metadata for each issuer.
    #[pallet::storage]
    #[pallet::getter(fn idty_cert_meta)]
    pub type StorageIdtyCertMeta<T: Config> =
        StorageMap<_, Twox64Concat, T::IdtyIndex, IdtyCertMeta<BlockNumberFor<T>>, ValueQuery>;

    /// The certifications for each receiver.
    #[pallet::storage]
    #[pallet::getter(fn certs_by_receiver)]
    pub type CertsByReceiver<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::IdtyIndex,
        Vec<(T::IdtyIndex, BlockNumberFor<T>)>,
        ValueQuery,
    >;

    /// The certifications that should expire at a given block.
    #[pallet::storage]
    #[pallet::getter(fn certs_removable_on)]
    pub type CertsRemovableOn<T: Config> = StorageMap<
        _,
        Twox64Concat,
        BlockNumberFor<T>,
        Vec<(T::IdtyIndex, T::IdtyIndex)>,
        OptionQuery,
    >;

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new certification was added.
        CertAdded {
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        },
        /// A certification was removed.
        CertRemoved {
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            expiration: bool,
        },
        /// A certification was renewed.
        CertRenewed {
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Issuer of a certification must have an identity
        OriginMustHaveAnIdentity,
        /// Identity cannot certify itself.
        CannotCertifySelf,
        /// Identity has already issued the maximum number of certifications.
        IssuedTooManyCert,
        /// Insufficient certifications received.
        NotEnoughCertReceived,
        /// Identity has issued a certification too recently.
        NotRespectCertPeriod,
        /// Can not add an already-existing cert
        CertAlreadyExists,
        /// Can not renew a non-existing cert
        CertDoesNotExist,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            Self::prune_certifications(n).saturating_add(T::WeightInfo::on_initialize())
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new certification.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_cert())]
        pub fn add_cert(
            origin: OriginFor<T>,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let issuer = Self::origin_to_index(origin)?;
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            Self::check_add_cert(issuer, receiver, block_number)?;
            Self::try_add_cert(block_number, issuer, receiver)?;
            Ok(().into())
        }

        /// Renew an existing certification.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::renew_cert())]
        pub fn renew_cert(
            origin: OriginFor<T>,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let issuer = Self::origin_to_index(origin)?;
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            Self::check_renew_cert(issuer, receiver, block_number)?;
            Self::try_renew_cert(block_number, issuer, receiver)?;
            Ok(().into())
        }

        /// Remove one certification given the issuer and the receiver.
        ///
        /// - `origin`: Must be `Root`.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::del_cert())]
        pub fn del_cert(
            origin: OriginFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::do_remove_cert(issuer, receiver, None);
            Ok(().into())
        }

        /// Remove all certifications received by an identity.
        ///
        /// - `origin`: Must be `Root`.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::remove_all_certs_received_by(CertsByReceiver::<T>::get(idty_index).len() as u32))]
        pub fn remove_all_certs_received_by(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let _ = Self::do_remove_all_certs_received_by(idty_index);
            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Perform removal of all certifications received by an identity.
        pub fn do_remove_all_certs_received_by(idty_index: T::IdtyIndex) -> Weight {
            let received_certs = CertsByReceiver::<T>::take(idty_index);
            for (receiver_received_count, (issuer, _)) in received_certs.iter().enumerate().rev() {
                let issuer_issued_count =
                    <StorageIdtyCertMeta<T>>::mutate_exists(issuer, |cert_meta_opt| {
                        let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                        cert_meta.issued_count = cert_meta.issued_count.saturating_sub(1);
                        cert_meta.issued_count
                    });
                T::OnRemovedCert::on_removed_cert(
                    *issuer,
                    issuer_issued_count,
                    idty_index,
                    receiver_received_count as u32,
                    false,
                );
                Self::deposit_event(Event::CertRemoved {
                    issuer: *issuer,
                    receiver: idty_index,
                    expiration: false,
                });
            }
            T::WeightInfo::do_remove_all_certs_received_by(received_certs.len() as u32)
        }

        /// Get the issuer index from the origin.
        pub fn origin_to_index(origin: OriginFor<T>) -> Result<T::IdtyIndex, DispatchError> {
            let who = ensure_signed(origin)?;
            T::IdtyAttr::idty_index(who).ok_or(Error::<T>::OriginMustHaveAnIdentity.into())
        }

        /// Add a certification without performing checks.
        ///
        /// This function is used during identity creation to add the first certification without
        /// validation checks.
        // The weight is approximated based on the worst-case scenario path.
        pub fn do_add_cert_checked(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            verify_rules: bool,
        ) -> DispatchResultWithPostInfo {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();

            if verify_rules {
                // only verify internal rules if asked
                Self::check_add_cert_internal(issuer, receiver, block_number)?;
            };

            Self::try_add_cert(block_number, issuer, receiver)?;
            Ok(().into())
        }

        /// Perform certification addition if it does not already exist, otherwise return `CertAlreadyExists`.
        // must be transactional
        fn try_add_cert(
            block_number: BlockNumberFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            // Write CertsRemovableOn
            let removable_on = block_number + T::ValidityPeriod::get();
            <CertsRemovableOn<T>>::append(removable_on, (issuer, receiver));

            // Write CertsByReceiver
            CertsByReceiver::<T>::mutate_exists(receiver, |maybe_issuers| {
                let issuers = maybe_issuers.get_or_insert(Vec::with_capacity(0));
                // cert does not exist, must be created
                if let Err(index) = issuers.binary_search_by(|(issuer_, _)| issuer_.cmp(&issuer)) {
                    issuers.insert(index, (issuer, removable_on));
                    Ok(())
                } else {
                    // cert exists, must be renewed instead
                    Err(Error::<T>::CertAlreadyExists)
                }
            })?;

            // Write StorageIdtyCertMeta for issuer
            let issuer_issued_count =
                StorageIdtyCertMeta::<T>::mutate(issuer, |issuer_idty_cert_meta| {
                    issuer_idty_cert_meta.issued_count =
                        issuer_idty_cert_meta.issued_count.saturating_add(1);
                    issuer_idty_cert_meta.next_issuable_on = block_number + T::CertPeriod::get();
                    issuer_idty_cert_meta.issued_count
                });

            // Write StorageIdtyCertMeta for receiver
            let receiver_received_count =
                <StorageIdtyCertMeta<T>>::mutate_exists(receiver, |cert_meta_opt| {
                    let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                    cert_meta.received_count = cert_meta.received_count.saturating_add(1);
                    cert_meta.received_count
                });

            // emit CertAdded event
            Self::deposit_event(Event::CertAdded { issuer, receiver });
            T::OnNewcert::on_new_cert(
                issuer,
                issuer_issued_count,
                receiver,
                receiver_received_count,
            );
            Ok(().into())
        }

        /// Perform certification renewal if it exists, otherwise return an error indicating `CertDoesNotExist`.
        // must be used in transactional context
        // (it can fail if certification does not exist after having modified state)
        fn try_renew_cert(
            block_number: BlockNumberFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            // Write CertsRemovableOn
            let removable_on = block_number + T::ValidityPeriod::get();
            <CertsRemovableOn<T>>::append(removable_on, (issuer, receiver));
            // Write CertsByReceiver
            CertsByReceiver::<T>::mutate_exists(receiver, |maybe_issuers| {
                let issuers = maybe_issuers.get_or_insert(Vec::with_capacity(0));
                // cert exists, can be renewed
                if let Ok(index) = issuers.binary_search_by(|(issuer_, _)| issuer_.cmp(&issuer)) {
                    issuers[index] = (issuer, removable_on);
                    Ok(())
                } else {
                    // cert does not exist, must be created
                    Err(Error::<T>::CertDoesNotExist)
                }
            })?;
            // Update next_issuable_on in StorageIdtyCertMeta for issuer
            StorageIdtyCertMeta::<T>::mutate(issuer, |issuer_idty_cert_meta| {
                issuer_idty_cert_meta.next_issuable_on = block_number + T::CertPeriod::get();
            });
            // emit CertRenewed event
            Self::deposit_event(Event::CertRenewed { issuer, receiver });
            Ok(().into())
        }

        /// Remove certifications that are due to expire on the given block.
        // (run at on_initialize step)
        fn prune_certifications(block_number: BlockNumberFor<T>) -> Weight {
            // See on initialize for the overhead weight accounting
            let mut weight = Weight::zero();

            if let Some(certs) = CertsRemovableOn::<T>::take(block_number) {
                for (issuer, receiver) in certs {
                    weight = weight.saturating_add(Self::do_remove_cert(
                        issuer,
                        receiver,
                        Some(block_number),
                    ));
                }
            }
            weight
        }

        /// Perform the certification removal.
        ///
        /// If a block number is provided, this function removes certifications only if they are still
        /// scheduled to expire at that block number.
        // This function is used because the unscheduling of certification expiry (#110) is not yet implemented.
        pub fn do_remove_cert(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            block_number_opt: Option<BlockNumberFor<T>>,
        ) -> Weight {
            let mut total_weight = Weight::zero();
            let mut removed = false;
            CertsByReceiver::<T>::mutate_exists(receiver, |issuers_opt| {
                let issuers = issuers_opt.get_or_insert(Vec::with_capacity(0));
                if let Ok(index) = issuers.binary_search_by(|(issuer_, _)| issuer_.cmp(&issuer)) {
                    if let Some(block_number) = block_number_opt {
                        if let Some((_, removable_on)) = issuers.get(index) {
                            // only remove cert if block number is matching
                            if *removable_on == block_number {
                                issuers.remove(index);
                                removed = true;
                            }
                        }
                    } else {
                        issuers.remove(index);
                        removed = true;
                    }
                } else {
                    total_weight += T::WeightInfo::do_remove_cert_noop();
                }
            });
            if removed {
                let issuer_issued_count =
                    <StorageIdtyCertMeta<T>>::mutate_exists(issuer, |cert_meta_opt| {
                        let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                        cert_meta.issued_count = cert_meta.issued_count.saturating_sub(1);
                        cert_meta.issued_count
                    });
                let receiver_received_count =
                    <StorageIdtyCertMeta<T>>::mutate_exists(receiver, |cert_meta_opt| {
                        let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                        cert_meta.received_count = cert_meta.received_count.saturating_sub(1);
                        cert_meta.received_count
                    });
                Self::deposit_event(Event::CertRemoved {
                    issuer,
                    receiver,
                    expiration: block_number_opt.is_some(),
                });
                T::OnRemovedCert::on_removed_cert(
                    issuer,
                    issuer_issued_count,
                    receiver,
                    receiver_received_count,
                    block_number_opt.is_some(),
                );
                // Pessimistic overhead estimation based on the worst path of a successfull
                // certificate removal to avoid multiplying benchmarks for every branching,
                // include the OnRemovedCert weight.
                total_weight.saturating_add(T::WeightInfo::do_remove_cert());
            }
            total_weight
        }

        /// Check if adding a certification is allowed.
        // 1. no self cert
        // 2. issuer received cert count
        // 3. issuer max emitted cert
        // 4. issuer cert period
        fn check_add_cert_internal(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            block_number: BlockNumberFor<T>,
        ) -> DispatchResult {
            // 1. Forbid self cert
            ensure!(issuer != receiver, Error::<T>::CannotCertifySelf);

            // 2. Verify rule MinReceivedCertToBeAbleToIssueCert
            // (this number can differ from the one necessary to be member)
            let issuer_idty_cert_meta = <StorageIdtyCertMeta<T>>::get(issuer);
            ensure!(
                issuer_idty_cert_meta.received_count
                    >= T::MinReceivedCertToBeAbleToIssueCert::get(),
                Error::<T>::NotEnoughCertReceived
            );

            // 3. Verify rule MaxByIssuer
            ensure!(
                issuer_idty_cert_meta.issued_count < T::MaxByIssuer::get(),
                Error::<T>::IssuedTooManyCert
            );

            // 4. Verify rule CertPeriod
            ensure!(
                block_number >= issuer_idty_cert_meta.next_issuable_on,
                Error::<T>::NotRespectCertPeriod
            );

            Ok(())
        }

        /// Check if adding a certification is allowed.
        // first internal checks
        // then external checks
        fn check_add_cert(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            block_number: BlockNumberFor<T>,
        ) -> DispatchResult {
            // internal checks
            Self::check_add_cert_internal(issuer, receiver, block_number)?;

            // --- then external checks
            // - issuer is member
            // - receiver is confirmed
            // - receiver is not revoked
            T::CheckCertAllowed::check_cert_allowed(issuer, receiver)?;

            Ok(())
        }

        /// Check if renewing a certification is allowed based.
        fn check_renew_cert(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            block_number: BlockNumberFor<T>,
        ) -> DispatchResult {
            Self::check_add_cert_internal(issuer, receiver, block_number)?;
            T::CheckCertAllowed::check_cert_allowed(issuer, receiver)?;
            Ok(())
        }
    }
}

// implement setting next_issuable_on for certification period
impl<T: Config> SetNextIssuableOn<BlockNumberFor<T>, T::IdtyIndex> for Pallet<T> {
    fn set_next_issuable_on(idty_index: T::IdtyIndex, next_issuable_on: BlockNumberFor<T>) {
        <StorageIdtyCertMeta<T>>::mutate_exists(idty_index, |cert_meta_opt| {
            let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
            cert_meta.next_issuable_on = next_issuable_on;
        });
    }
}
