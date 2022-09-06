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

pub mod traits;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::*;

use crate::traits::*;
use codec::Codec;
use frame_support::traits::StorageVersion;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::{fmt::Debug, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Convert, Saturating};
    use sp_std::collections::btree_map::BTreeMap;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        #[pallet::constant]
        /// Minimum duration between two certifications issued by the same issuer
        type CertPeriod: Get<Self::BlockNumber>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
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
        /// Something that give the owner key of an identity
        type OwnerKeyOf: Convert<Self::IdtyIndex, Option<Self::AccountId>>;
        ///
        type IsCertAllowed: IsCertAllowed<Self::IdtyIndex>;
        #[pallet::constant]
        /// Maximum number of active certifications by issuer
        type MaxByIssuer: Get<u32>;
        /// Minimum number of certifications that must be received to be able to issue
        /// certifications.
        type MinReceivedCertToBeAbleToIssueCert: Get<u32>;
        /// Handler for NewCert event
        type OnNewcert: OnNewcert<Self::IdtyIndex>;
        /// Handler for Removed event
        type OnRemovedCert: OnRemovedCert<Self::IdtyIndex>;
        #[pallet::constant]
        /// Duration of validity of a certification
        type ValidityPeriod: Get<Self::BlockNumber>;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    #[allow(clippy::type_complexity)]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub apply_cert_period_at_genesis: bool,
        pub certs_by_receiver:
            BTreeMap<T::IdtyIndex, BTreeMap<T::IdtyIndex, Option<T::BlockNumber>>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                apply_cert_period_at_genesis: false,
                certs_by_receiver: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            let mut cert_meta_by_issuer =
                BTreeMap::<T::IdtyIndex, IdtyCertMeta<T::BlockNumber>>::new();
            let mut certs_removable_on =
                BTreeMap::<T::BlockNumber, Vec<(T::IdtyIndex, T::IdtyIndex)>>::new();
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
                        received_count: issuers.len() as u32,
                    });

                let mut issuers_: Vec<_> = Vec::with_capacity(issuers.len());
                for (issuer, maybe_removable_on) in issuers {
                    // Count issued certs
                    cert_meta_by_issuer
                        .entry(*issuer)
                        .or_insert(IdtyCertMeta {
                            issued_count: 0,
                            next_issuable_on: sp_runtime::traits::Zero::zero(),
                            received_count: issuers.len() as u32,
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
                CertsByReceiver::<T, I>::insert(receiver, issuers_);
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
                StorageIdtyCertMeta::<T, I>::insert(issuer, cert_meta);
            }
            // Write storage StorageCertsRemovableOn
            for (removable_on, certs) in certs_removable_on {
                StorageCertsRemovableOn::<T, I>::insert(removable_on, certs);
            }
        }
    }

    // STORAGE //

    /// Certifications metada by issuer
    #[pallet::storage]
    #[pallet::getter(fn idty_cert_meta)]
    pub type StorageIdtyCertMeta<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::IdtyIndex, IdtyCertMeta<T::BlockNumber>, ValueQuery>;

    /// Certifications by receiver
    #[pallet::storage]
    #[pallet::getter(fn certs_by_receiver)]
    pub type CertsByReceiver<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::IdtyIndex, Vec<(T::IdtyIndex, T::BlockNumber)>, ValueQuery>;

    /// Certifications removable on
    #[pallet::storage]
    #[pallet::getter(fn certs_removable_on)]
    pub type StorageCertsRemovableOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<(T::IdtyIndex, T::IdtyIndex)>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// New certification
        /// [issuer, issuer_issued_count, receiver, receiver_received_count]
        NewCert {
            issuer: T::IdtyIndex,
            issuer_issued_count: u32,
            receiver: T::IdtyIndex,
            receiver_received_count: u32,
        },
        /// Removed certification
        /// [issuer, issuer_issued_count, receiver, receiver_received_count, expiration]
        RemovedCert {
            issuer: T::IdtyIndex,
            issuer_issued_count: u32,
            receiver: T::IdtyIndex,
            receiver_received_count: u32,
            expiration: bool,
        },
        /// Renewed certification
        /// [issuer, receiver]
        RenewedCert {
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// An identity cannot certify itself
        CannotCertifySelf,
        /// Certification non autorisée
        CertNotAllowed,
        /// This identity has already issued the maximum number of certifications
        IssuedTooManyCert,
        /// Issuer not found
        IssuerNotFound,
        /// Not enough certifications received
        NotEnoughCertReceived,
        /// This identity has already issued a certification too recently
        NotRespectCertPeriod,
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            Self::prune_certifications(n)
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        #[pallet::weight(1_000_000_000)]
        pub fn force_add_cert(
            origin: OriginFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            verify_rules: bool,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            // Forbid self cert
            ensure!(issuer != receiver, Error::<T, I>::CannotCertifySelf);

            let block_number = frame_system::pallet::Pallet::<T>::block_number();

            if verify_rules {
                // Verify rule MinReceivedCertToBeAbleToIssueCert
                let issuer_idty_cert_meta = StorageIdtyCertMeta::<T, I>::get(issuer);
                ensure!(
                    issuer_idty_cert_meta.received_count
                        >= T::MinReceivedCertToBeAbleToIssueCert::get(),
                    Error::<T, I>::NotEnoughCertReceived
                );

                // Verify rule MaxByIssuer
                ensure!(
                    issuer_idty_cert_meta.issued_count < T::MaxByIssuer::get(),
                    Error::<T, I>::IssuedTooManyCert
                );

                // Verify rule CertPeriod
                ensure!(
                    block_number >= issuer_idty_cert_meta.next_issuable_on,
                    Error::<T, I>::NotRespectCertPeriod
                );
            };

            Self::do_add_cert(block_number, issuer, receiver)
        }
        /// Add a new certification or renew an existing one
        ///
        /// - `receiver`: the account receiving the certification from the origin
        ///
        /// The origin must be allow to certify.
        #[pallet::weight(1_000_000_000)]
        pub fn add_cert(
            origin: OriginFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Forbid self cert
            ensure!(issuer != receiver, Error::<T, I>::CannotCertifySelf);

            // Verify caller ownership
            let issuer_owner_key =
                T::OwnerKeyOf::convert(issuer).ok_or(Error::<T, I>::IssuerNotFound)?;
            ensure!(issuer_owner_key == who, DispatchError::BadOrigin);

            // Verify compatibility with other pallets state
            ensure!(
                T::IsCertAllowed::is_cert_allowed(issuer, receiver),
                Error::<T, I>::CertNotAllowed
            );

            // Verify rule MinReceivedCertToBeAbleToIssueCert
            let issuer_idty_cert_meta = <StorageIdtyCertMeta<T, I>>::get(issuer);
            ensure!(
                issuer_idty_cert_meta.received_count
                    >= T::MinReceivedCertToBeAbleToIssueCert::get(),
                Error::<T, I>::NotEnoughCertReceived
            );

            // Verify rule MaxByIssuer
            ensure!(
                issuer_idty_cert_meta.issued_count < T::MaxByIssuer::get(),
                Error::<T, I>::IssuedTooManyCert
            );

            // Verify rule CertPeriod
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            ensure!(
                block_number >= issuer_idty_cert_meta.next_issuable_on,
                Error::<T, I>::NotRespectCertPeriod
            );

            Self::do_add_cert(block_number, issuer, receiver)
        }

        #[pallet::weight(1_000_000_000)]
        pub fn del_cert(
            origin: OriginFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::remove_cert_inner(issuer, receiver, None);
            Ok(().into())
        }

        #[pallet::weight(1_000_000_000)]
        pub fn remove_all_certs_received_by(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            for (issuer, _) in CertsByReceiver::<T, I>::get(idty_index) {
                Self::remove_cert_inner(issuer, idty_index, None);
            }
            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        fn do_add_cert(
            block_number: T::BlockNumber,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            // Write StorageCertsRemovableOn
            let removable_on = block_number + T::ValidityPeriod::get();
            <StorageCertsRemovableOn<T, I>>::append(removable_on, (issuer, receiver));

            // Write CertsByReceiver
            let mut created = false;
            CertsByReceiver::<T, I>::mutate_exists(receiver, |maybe_issuers| {
                let issuers = maybe_issuers.get_or_insert(Vec::with_capacity(0));
                if let Err(index) = issuers.binary_search_by(|(issuer_, _)| issuer.cmp(issuer_)) {
                    issuers.insert(index, (issuer, removable_on));
                    created = true;
                }
            });

            if created {
                // Write StorageIdtyCertMeta for issuer
                let issuer_issued_count =
                    StorageIdtyCertMeta::<T, I>::mutate(issuer, |issuer_idty_cert_meta| {
                        issuer_idty_cert_meta.issued_count =
                            issuer_idty_cert_meta.issued_count.saturating_add(1);
                        issuer_idty_cert_meta.next_issuable_on =
                            block_number + T::CertPeriod::get();
                        issuer_idty_cert_meta.issued_count
                    });

                // Write StorageIdtyCertMeta for receiver
                let receiver_received_count =
                    <StorageIdtyCertMeta<T, I>>::mutate_exists(receiver, |cert_meta_opt| {
                        let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                        cert_meta.received_count = cert_meta.received_count.saturating_add(1);
                        cert_meta.received_count
                    });

                Self::deposit_event(Event::NewCert {
                    issuer,
                    issuer_issued_count,
                    receiver,
                    receiver_received_count,
                });
                T::OnNewcert::on_new_cert(
                    issuer,
                    issuer_issued_count,
                    receiver,
                    receiver_received_count,
                );
            } else {
                Self::deposit_event(Event::RenewedCert { issuer, receiver });
            }

            Ok(().into())
        }
        fn prune_certifications(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0;

            if let Some(certs) = StorageCertsRemovableOn::<T, I>::take(block_number) {
                for (issuer, receiver) in certs {
                    total_weight += Self::remove_cert_inner(issuer, receiver, Some(block_number));
                }
            }

            total_weight
        }
        fn remove_cert_inner(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            block_number_opt: Option<T::BlockNumber>,
        ) -> Weight {
            let mut total_weight: Weight = 0;
            let mut removed = false;
            CertsByReceiver::<T, I>::mutate_exists(receiver, |issuers_opt| {
                let issuers = issuers_opt.get_or_insert(Vec::with_capacity(0));
                if let Ok(index) = issuers.binary_search_by(|(issuer_, _)| issuer.cmp(issuer_)) {
                    if let Some(block_number) = block_number_opt {
                        if let Some((_, removable_on)) = issuers.get(index) {
                            if *removable_on == block_number {
                                issuers.remove(index);
                                removed = true;
                            }
                        }
                    } else {
                        issuers.remove(index);
                        removed = true;
                    }
                }
            });
            if removed {
                let issuer_issued_count =
                    <StorageIdtyCertMeta<T, I>>::mutate_exists(issuer, |cert_meta_opt| {
                        let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                        cert_meta.issued_count = cert_meta.issued_count.saturating_sub(1);
                        cert_meta.issued_count
                    });
                let receiver_received_count =
                    <StorageIdtyCertMeta<T, I>>::mutate_exists(receiver, |cert_meta_opt| {
                        let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                        cert_meta.received_count = cert_meta.received_count.saturating_sub(1);
                        cert_meta.received_count
                    });
                Self::deposit_event(Event::RemovedCert {
                    issuer,
                    issuer_issued_count,
                    receiver,
                    receiver_received_count,
                    expiration: block_number_opt.is_some(),
                });
                total_weight += T::OnRemovedCert::on_removed_cert(
                    issuer,
                    issuer_issued_count,
                    receiver,
                    receiver_received_count,
                    block_number_opt.is_some(),
                );
            }
            total_weight
        }
    }
}

impl<T: Config<I>, I: 'static> SetNextIssuableOn<T::BlockNumber, T::IdtyIndex> for Pallet<T, I> {
    fn set_next_issuable_on(
        idty_index: T::IdtyIndex,
        next_issuable_on: T::BlockNumber,
    ) -> frame_support::pallet_prelude::Weight {
        <StorageIdtyCertMeta<T, I>>::mutate_exists(idty_index, |cert_meta_opt| {
            let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
            cert_meta.next_issuable_on = next_issuable_on;
        });
        0
    }
}
