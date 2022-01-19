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
    use sp_runtime::traits::Saturating;
    use sp_std::collections::btree_map::BTreeMap;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// Origin allowed to add a certification
        type AddCertOrigin: EnsureOrigin<(Self::Origin, Self::IdtyIndex, Self::IdtyIndex)>;
        #[pallet::constant]
        /// Minimum duration between two certifications issued by the same issuer
        type CertPeriod: Get<Self::BlockNumber>;
        /// Origin allowed to delete a certification
        type DelCertOrigin: EnsureOrigin<(Self::Origin, Self::IdtyIndex, Self::IdtyIndex)>;
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
        #[pallet::constant]
        /// Maximum number of active certifications by issuer
        type MaxByIssuer: Get<u8>;
        /// Handler for NewCert event
        type OnNewcert: OnNewcert<Self::IdtyIndex>;
        /// Handler for Removed event
        type OnRemovedCert: OnRemovedCert<Self::IdtyIndex>;
        #[pallet::constant]
        /// Duration after which a certification is renewable
        type CertRenewablePeriod: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Duration of validity of a certification
        type ValidityPeriod: Get<Self::BlockNumber>;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub apply_cert_period_at_genesis: bool,
        pub certs_by_issuer: BTreeMap<T::IdtyIndex, BTreeMap<T::IdtyIndex, T::BlockNumber>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                apply_cert_period_at_genesis: false,
                certs_by_issuer: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            let mut cert_meta_by_issuer =
                BTreeMap::<T::IdtyIndex, IdtyCertMeta<T::BlockNumber>>::new();
            let mut certs_by_receiver = BTreeMap::<T::IdtyIndex, Vec<T::IdtyIndex>>::new();
            for (issuer, receivers) in &self.certs_by_issuer {
                assert!(
                    !receivers.contains_key(issuer),
                    "Identity cannot certify it-self."
                );
                assert!(
                    !receivers.len() >= T::MaxByIssuer::get() as usize,
                    "Identity n°{:?} exceed MaxByIssuer.",
                    issuer
                );

                cert_meta_by_issuer.insert(
                    *issuer,
                    IdtyCertMeta {
                        issued_count: receivers.len() as u8,
                        next_issuable_on: sp_runtime::traits::Zero::zero(),
                        received_count: 0,
                    },
                );
                for receiver in receivers.keys() {
                    certs_by_receiver
                        .entry(*receiver)
                        .or_default()
                        .push(*issuer);
                }
            }

            // Write StorageCertsByReceiver
            for (receiver, mut issuers) in certs_by_receiver {
                cert_meta_by_issuer
                    .entry(receiver)
                    .and_modify(|cert_meta| cert_meta.received_count = issuers.len() as u32);
                issuers.sort();
                <StorageCertsByReceiver<T, I>>::insert(receiver, issuers);
            }
            // Write StorageCertsByIssuer
            let mut certs_removable_on =
                BTreeMap::<T::BlockNumber, Vec<(T::IdtyIndex, T::IdtyIndex)>>::new();
            for (issuer, receivers) in &self.certs_by_issuer {
                for (receiver, removable_on) in receivers {
                    certs_removable_on
                        .entry(*removable_on)
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
                    let renewable_on = removable_on.saturating_sub(
                        T::ValidityPeriod::get().saturating_sub(T::CertRenewablePeriod::get()),
                    );

                    <StorageCertsByIssuer<T, I>>::insert(
                        issuer,
                        receiver,
                        CertValue {
                            renewable_on,
                            removable_on: *removable_on,
                        },
                    );
                }
            }
            // Write StorageIdtyCertMeta
            for (issuer, cert_meta) in cert_meta_by_issuer {
                <StorageIdtyCertMeta<T, I>>::insert(issuer, cert_meta);
            }
            // Write storage StorageCertsRemovableOn
            for (removable_on, certs) in certs_removable_on {
                <StorageCertsRemovableOn<T, I>>::insert(removable_on, certs);
            }
        }
    }

    // STORAGE //

    /// Certifications metada by issuer
    #[pallet::storage]
    #[pallet::getter(fn idty_cert_meta)]
    pub type StorageIdtyCertMeta<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::IdtyIndex, IdtyCertMeta<T::BlockNumber>, OptionQuery>;

    /// Certifications by issuer
    #[pallet::storage]
    #[pallet::getter(fn cert)]
    /// Certifications by issuer
    pub(super) type StorageCertsByIssuer<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Identity,
        T::IdtyIndex,
        Identity,
        T::IdtyIndex,
        CertValue<T::BlockNumber>,
        OptionQuery,
        GetDefault,
        ConstU32<4_000_000_000>,
    >;

    /// Certifications by receiver
    #[pallet::storage]
    #[pallet::getter(fn certs_by_receiver)]
    pub type StorageCertsByReceiver<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::IdtyIndex, Vec<T::IdtyIndex>, OptionQuery>;

    /// Certifications removable on
    #[pallet::storage]
    #[pallet::getter(fn certs_removable_on)]
    pub type StorageCertsRemovableOn<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        T::BlockNumber,
        Vec<(T::IdtyIndex, T::IdtyIndex)>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// New certification
        /// [issuer, issuer_issued_count, receiver, receiver_received_count]
        NewCert {
            issuer: T::IdtyIndex,
            issuer_issued_count: u8,
            receiver: T::IdtyIndex,
            receiver_received_count: u32,
        },
        /// Removed certification
        /// [issuer, issuer_issued_count, receiver, receiver_received_count, expiration]
        RemovedCert {
            issuer: T::IdtyIndex,
            issuer_issued_count: u8,
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
        /// An identity must receive certifications before it can issue them.
        IdtyMustReceiveCertsBeforeCanIssue,
        /// This identity has already issued the maximum number of certifications
        IssuedTooManyCert,
        /// This certification has already been issued or renewed recently
        NotRespectRenewablePeriod,
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
        #[pallet::weight(0)]
        pub fn add_cert(
            origin: OriginFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            T::AddCertOrigin::ensure_origin((origin, issuer, receiver))?;
            frame_support::runtime_print!("add_cert({:?}, {:?}): origin OK", issuer, receiver);

            let block_number = frame_system::pallet::Pallet::<T>::block_number();

            let (create, issuer_issued_count) = if let Ok(mut issuer_idty_cert_meta) =
                <StorageIdtyCertMeta<T, I>>::try_get(issuer)
            {
                // Verify rules CertPeriod and MaxByIssuer
                frame_support::runtime_print!(
                    "add_cert({:?}, {:?}): Verify rules CertPeriod and MaxByIssuer",
                    issuer,
                    receiver
                );
                if issuer_idty_cert_meta.next_issuable_on > block_number {
                    return Err(Error::<T, I>::NotRespectCertPeriod.into());
                } else if issuer_idty_cert_meta.issued_count >= T::MaxByIssuer::get() {
                    return Err(Error::<T, I>::IssuedTooManyCert.into());
                }

                // Verify rule CertRenewablePeriod
                frame_support::runtime_print!(
                    "add_cert({:?}, {:?}): Verify rule CertRenewablePeriod",
                    issuer,
                    receiver
                );
                let create = if let Ok(CertValue { renewable_on, .. }) =
                    <StorageCertsByIssuer<T, I>>::try_get(issuer, receiver)
                {
                    if renewable_on > block_number {
                        return Err(Error::<T, I>::NotRespectRenewablePeriod.into());
                    }
                    false
                } else {
                    true
                };

                // Write StorageIdtyCertMeta for issuer
                issuer_idty_cert_meta.issued_count =
                    issuer_idty_cert_meta.issued_count.saturating_add(1);
                let issuer_issued_count = issuer_idty_cert_meta.issued_count;
                issuer_idty_cert_meta.next_issuable_on = block_number + T::CertPeriod::get();
                <StorageIdtyCertMeta<T, I>>::insert(issuer, issuer_idty_cert_meta);

                (create, issuer_issued_count)
            } else {
                // An identity must receive certifications before it can issue them.
                return Err(Error::<T, I>::IdtyMustReceiveCertsBeforeCanIssue.into());
            };

            // Write StorageIdtyCertMeta for receiver
            frame_support::runtime_print!(
                "add_cert({:?}, {:?}): Write StorageIdtyCertMeta for receiver",
                issuer,
                receiver
            );
            let receiver_received_count =
                <StorageIdtyCertMeta<T, I>>::mutate_exists(receiver, |cert_meta_opt| {
                    let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                    cert_meta.received_count = cert_meta.received_count.saturating_add(1);
                    cert_meta.received_count
                });

            // Write StorageCertsRemovableOn and StorageCertsByIssuer
            let cert_value = CertValue {
                renewable_on: block_number + T::CertRenewablePeriod::get(),
                removable_on: block_number + T::ValidityPeriod::get(),
            };
            <StorageCertsRemovableOn<T, I>>::append(cert_value.removable_on, (issuer, receiver));
            <StorageCertsByIssuer<T, I>>::insert(issuer, receiver, cert_value);

            if create {
                // Write StorageCertsByReceiver
                <StorageCertsByReceiver<T, I>>::mutate_exists(receiver, |issuers_opt| {
                    let issuers = issuers_opt.get_or_insert(Vec::with_capacity(0));
                    if let Err(index) = issuers.binary_search(&issuer) {
                        issuers.insert(index, issuer);
                    }
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

        #[pallet::weight(0)]
        pub fn del_cert(
            origin: OriginFor<T>,
            issuer: T::IdtyIndex,
            receiver: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            T::DelCertOrigin::ensure_origin((origin, issuer, receiver))?;
            Self::remove_cert_inner(issuer, receiver, None);
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn remove_all_certs_received_by(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            if let Ok(issuers) = <StorageCertsByReceiver<T, I>>::try_get(idty_index) {
                for issuer in issuers {
                    Self::remove_cert_inner(issuer, idty_index, None);
                }
            }
            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        fn prune_certifications(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0;

            use frame_support::storage::generator::StorageMap as _;
            if let Some(certs) = StorageCertsRemovableOn::<T, I>::from_query_to_optional_value(
                StorageCertsRemovableOn::<T, I>::take(block_number),
            ) {
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
            <StorageCertsByIssuer<T, I>>::mutate_exists(issuer, receiver, |cert_val_opt| {
                if let Some(cert_val) = cert_val_opt {
                    if Some(cert_val.removable_on) == block_number_opt || block_number_opt.is_none()
                    {
                        removed = true;
                    }
                }
                if removed {
                    cert_val_opt.take();
                }
            });
            if removed {
                <StorageCertsByReceiver<T, I>>::mutate_exists(receiver, |issuers_opt| {
                    let issuers = issuers_opt.get_or_insert(Vec::with_capacity(0));
                    if let Ok(index) = issuers.binary_search(&issuer) {
                        issuers.remove(index);
                    }
                });
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
