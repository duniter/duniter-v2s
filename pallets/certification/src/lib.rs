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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod traits;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use crate::traits::*;
use codec::Codec;
use frame_support::pallet_prelude::*;
use frame_support::traits::StorageVersion;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::{fmt::Debug, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Convert, Saturating};
    use sp_std::collections::btree_map::BTreeMap;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        #[pallet::constant]
        /// Minimum duration between two certifications issued by the same issuer
        type CertPeriod: Get<Self::BlockNumber>;
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
        type CheckCertAllowed: CheckCertAllowed<Self::IdtyIndex>;
        #[pallet::constant]
        /// Maximum number of active certifications by issuer
        type MaxByIssuer: Get<u32>;
        /// Minimum number of certifications that must be received to be able to issue
        /// certifications.
        #[pallet::constant]
        type MinReceivedCertToBeAbleToIssueCert: Get<u32>;
        /// Handler for NewCert event
        type OnNewcert: OnNewcert<Self::IdtyIndex>;
        /// Handler for Removed event
        type OnRemovedCert: OnRemovedCert<Self::IdtyIndex>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
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

    // EVENTS //

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

    // ERRORS //

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// An identity cannot certify itself
        CannotCertifySelf,
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
            Self::prune_certifications(n).saturating_add(T::WeightInfo::on_initialize())
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Add a new certification or renew an existing one
        ///
        /// - `receiver`: the account receiving the certification from the origin
        ///
        /// The origin must be allow to certify.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_cert())]
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
            T::CheckCertAllowed::check_cert_allowed(issuer, receiver)?;

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

        /// remove a certification (only root)
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

        /// remove all certifications received by an identity (only root)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::remove_all_certs_received_by(CertsByReceiver::<T, I>::get(idty_index).len() as u32))]
        pub fn remove_all_certs_received_by(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            for (issuer, _) in CertsByReceiver::<T, I>::get(idty_index) {
                Self::do_remove_cert(issuer, idty_index, None);
            }
            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// add a certification without checks
        pub fn do_add_cert_checked(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            verify_rules: bool,
        ) -> DispatchResultWithPostInfo {
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
        /// perform cert addition or renewal
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
                match issuers.binary_search_by(|(issuer_, _)| issuer_.cmp(&issuer)) {
                    // cert exists, must be renewed
                    Ok(index) => {
                        issuers[index] = (issuer, removable_on);
                    }
                    // cert does not exist, must be created
                    Err(index) => {
                        issuers.insert(index, (issuer, removable_on));
                        created = true;
                    }
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

                // emit NewCert event
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
                // Update next_issuable_on in StorageIdtyCertMeta for issuer
                StorageIdtyCertMeta::<T, I>::mutate(issuer, |issuer_idty_cert_meta| {
                    issuer_idty_cert_meta.next_issuable_on = block_number + T::CertPeriod::get();
                });
                // emit RenewedCert event
                Self::deposit_event(Event::RenewedCert { issuer, receiver });
            }

            Ok(().into())
        }
        /// remove the certifications due to expire on the given block
        // (run at on_initialize step)
        fn prune_certifications(block_number: T::BlockNumber) -> Weight {
            // See on initialize for the overhead weight accounting
            let mut total_weight = Weight::zero();

            if let Some(certs) = StorageCertsRemovableOn::<T, I>::take(block_number) {
                for (issuer, receiver) in certs {
                    total_weight += Self::do_remove_cert(issuer, receiver, Some(block_number));
                }
            }

            total_weight
        }
        /// perform the certification removal
        /// if block number is given only remove cert if still set to expire at this block number
        pub fn do_remove_cert(
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
            block_number_opt: Option<T::BlockNumber>,
        ) -> Weight {
            let mut total_weight = Weight::zero();
            let mut removed = false;
            CertsByReceiver::<T, I>::mutate_exists(receiver, |issuers_opt| {
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
                // Should always return Weight::zero
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
    }
}

// implement setting next_issuable_on for certification period
impl<T: Config<I>, I: 'static> SetNextIssuableOn<T::BlockNumber, T::IdtyIndex> for Pallet<T, I> {
    fn set_next_issuable_on(idty_index: T::IdtyIndex, next_issuable_on: T::BlockNumber) {
        <StorageIdtyCertMeta<T, I>>::mutate_exists(idty_index, |cert_meta_opt| {
            let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
            cert_meta.next_issuable_on = next_issuable_on;
        });
    }
}
