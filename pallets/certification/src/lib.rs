// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
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

#[cfg(test)]
mod mock;

pub use pallet::*;

use crate::traits::*;
use codec::Codec;
use sp_runtime::traits::{AtLeast32BitUnsigned, Zero};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::fmt::Debug;

pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    //use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    pub trait Config<I: Instance = DefaultInstance>: frame_system::Config {
        /// Origin allowed to add a certification
        type AddCertOrigin: EnsureOrigin<(Self::Origin, Self::IdtyIndex, Self::IdtyIndex)>;
        /// Minimum duration between two certifications issued by the same issuer
        type CertPeriod: Get<Self::BlockNumber>;
        /// Origin allowed to delete a certification
        type DelCertOrigin: EnsureOrigin<(Self::Origin, Self::IdtyIndex, Self::IdtyIndex)>;
        /// The overarching event type.
        type Event: From<Event<Self, I>> + Into<<Self as frame_system::Config>::Event>;
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
        /// Maximum number of active certifications by issuer
        type MaxByIssuer: Get<u8>;
        /// Handler for NewCert event
        type OnNewcert: OnNewcert<Self::IdtyIndex>;
        /// Handler for Removed event
        type OnRemovedCert: OnRemovedCert<Self::IdtyIndex>;
        /// Duration after which a certification is renewable
        type RenewablePeriod: Get<Self::BlockNumber>;
        /// Duration of validity of a certification
        type ValidityPeriod: Get<Self::BlockNumber>;
    }

    frame_support::decl_event! {
        pub enum Event<T, I=DefaultInstance> where
            <T as Config<I>>::IdtyIndex,
        {
            /// New certification
            /// \[issuer, issuer_issued_count, receiver, receiver_received_count\]
            NewCert(IdtyIndex,u8,  IdtyIndex, u32),
            /// Removed certification
            /// \[issuer, issuer_issued_count, receiver, receiver_received_count, expiration\]
            RemovedCert(IdtyIndex, u8, IdtyIndex, u32, bool),
            /// Renewed certification
            /// \[issuer, receiver\]
            RenewedCert(IdtyIndex, IdtyIndex),
        }
    }

    frame_support::decl_error! {
        pub enum Error for Module<T: Config<I>, I: Instance> {
            /// An identity must receive certifications before it can issue them.
            IdtyMustReceiveCertsBeforeCanIssue,
            /// This identity has already issued the maximum number of certifications
            IssuedTooManyCert,
            /// This certification has already been issued or renewed recently
            NotRespectRenewablePeriod,
            /// This identity has already issued a certification too recently
            NotRespectCertPeriod,
        }
    }

    // STORAGE //

    // A value placed in storage that represents the current version of the Balances storage.
    // This value is used by the `on_runtime_upgrade` logic to determine whether we run
    // storage migration logic. This should match directly with the semantic versions of the Rust crate.
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub enum Releases {
        V1_0_0,
    }
    impl Default for Releases {
        fn default() -> Self {
            Releases::V1_0_0
        }
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub struct CertValue<T: Config<I>, I: Instance> {
        chainable_on: T::BlockNumber,
        removable_on: T::BlockNumber,
        phantom: PhantomData<I>,
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub struct IdtyCertMeta<T: Config<I>, I: Instance> {
        issued_count: u8,
        next_issuable_on: T::BlockNumber,
        received_count: u32,
        phantom: PhantomData<I>,
    }
    impl<T: Config<I>, I: Instance> Default for IdtyCertMeta<T, I> {
        fn default() -> Self {
            Self {
                issued_count: 0,
                next_issuable_on: T::BlockNumber::zero(),
                received_count: 0,
                phantom: PhantomData,
            }
        }
    }

    frame_support::decl_storage! {
        trait Store for Module<T: Config<I>, I: Instance=DefaultInstance> as Certification {
            /// Storage version of the pallet.
            StorageVersion get(fn storage_version): Releases;
            /// Certifications by issuer
            pub StorageIdtyCertMeta get(fn certs_by_issuer):
            map hasher(twox_64_concat) T::IdtyIndex => IdtyCertMeta<T, I> = IdtyCertMeta {
                issued_count: 0,
                next_issuable_on: T::BlockNumber::zero(),
                received_count: 0,
                phantom: PhantomData,
            };
            pub StorageCertsByIssuer get(fn cert):
                double_map hasher(identity) T::IdtyIndex, hasher(identity) T::IdtyIndex
                => Option<CertValue<T, I>>;
            /// Certifications by receiver
            pub StorageCertsByReceiver get(fn certs_by_receiver):
            map hasher(twox_64_concat) T::IdtyIndex => Vec<T::IdtyIndex>;
            /// Certifications removable on
            pub StorageCertsRemovableOn get(fn certs_removable_on):
            map hasher(twox_64_concat) T::BlockNumber => Vec<(T::IdtyIndex, T::IdtyIndex)>;
        }
        add_extra_genesis {
            config(phantom): sp_std::marker::PhantomData<I>;
            config(certs_by_issuer): BTreeMap<T::IdtyIndex, BTreeSet<T::IdtyIndex>>;
            build(|config| {
                let mut cert_meta_by_issuer = BTreeMap::<T::IdtyIndex, IdtyCertMeta<T, I>>::new();
                let mut certs_by_receiver = BTreeMap::<T::IdtyIndex, Vec<T::IdtyIndex>>::new();
                for (issuer, receivers) in &config.certs_by_issuer {
                    assert!(!receivers.contains(issuer), "Identity cannot tcertify it-self.");
                    assert!(!receivers.len() <= T::MaxByIssuer::get() as usize, "Identity n°{:?} exceed MaxByIssuer.", issuer);

                    cert_meta_by_issuer.insert(*issuer, IdtyCertMeta {
                        issued_count: receivers.len() as u8,
                        next_issuable_on: T::CertPeriod::get(),
                        received_count: 0,
                        phantom: PhantomData,
                    });
                    for receiver in receivers {
                        certs_by_receiver.entry(*receiver).or_default().push(*issuer);
                    }
                }

                <StorageVersion<I>>::put(Releases::V1_0_0);
                // Write StorageCertsByReceiver
                for (receiver, issuers) in certs_by_receiver {
                    cert_meta_by_issuer.entry(receiver).and_modify(|cert_meta| cert_meta.received_count = issuers.len() as u32);
                    <StorageCertsByReceiver<T, I>>::insert(receiver, issuers);
                }
                // Write StorageIdtyCertMeta
                for (issuer, cert_meta) in cert_meta_by_issuer {
                    <StorageIdtyCertMeta<T, I>>::insert(issuer, cert_meta);
                }
                // Write StorageCertsByIssuer && StorageCertsRemovableOn
                let mut all_couples = Vec::new();
                for (issuer, receivers) in &config.certs_by_issuer {
                    for receiver in receivers {
                        all_couples.push((*issuer, *receiver));
                        <StorageCertsByIssuer<T, I>>::insert(issuer, receiver, CertValue {
                            chainable_on: T::RenewablePeriod::get(),
                            removable_on: T::ValidityPeriod::get(),
                            phantom: PhantomData,
                        });
                    }
                }
                <StorageCertsRemovableOn<T, I>>::insert(T::ValidityPeriod::get(), all_couples);
            });
        }
    }

    // CALLS //

    frame_support::decl_module! {
        pub struct Module<T: Config<I>, I: Instance=DefaultInstance> for enum Call where origin: <T as frame_system::Config>::Origin {
            type Error = Error<T, I>;

            fn deposit_event() = default;

            fn on_initialize(n: T::BlockNumber) -> Weight {
                Self::prune_certifications(n)
            }

            #[weight = 0]
            pub fn add_cert(origin, issuer: T::IdtyIndex, receiver: T::IdtyIndex) {
                T::AddCertOrigin::ensure_origin((origin, issuer, receiver))?;

                let block_number = frame_system::pallet::Pallet::<T>::block_number();

                let (create, issuer_issued_count) = if let Ok(mut issuer_idty_cert_meta) = <StorageIdtyCertMeta<T, I>>::try_get(issuer) {
                    // Verify rules CertPeriod and MaxByIssuer
                    if issuer_idty_cert_meta.next_issuable_on > block_number {
                        return Err(Error::<T, I>::NotRespectCertPeriod.into());
                    } else if issuer_idty_cert_meta.issued_count >= T::MaxByIssuer::get() {
                        return Err(Error::<T, I>::IssuedTooManyCert.into());
                    }

                    // Verify rule RenewablePeriod
                    let create = if let Ok(CertValue { chainable_on, .. }) = <StorageCertsByIssuer<T, I>>::try_get(issuer, receiver) {
                        if chainable_on > block_number {
                            return Err(Error::<T, I>::NotRespectRenewablePeriod.into());
                        }
                        false
                    } else {
                        true
                    };

                    // Write StorageIdtyCertMeta for issuer
                    issuer_idty_cert_meta.issued_count = issuer_idty_cert_meta.issued_count.saturating_add(1);
                    let issuer_issued_count = issuer_idty_cert_meta.issued_count;
                    issuer_idty_cert_meta.next_issuable_on = block_number + T::CertPeriod::get();
                    <StorageIdtyCertMeta<T, I>>::insert(issuer, issuer_idty_cert_meta);

                    (create, issuer_issued_count)
                } else {
                    // An identity must receive certifications before it can issue them.
                    return Err(Error::<T, I>::IdtyMustReceiveCertsBeforeCanIssue.into());
                };

                // Write StorageIdtyCertMeta for receiver
                let receiver_received_count = <StorageIdtyCertMeta<T, I>>::mutate_exists(receiver, |cert_meta_opt| {
                    let cert_meta = cert_meta_opt.get_or_insert(IdtyCertMeta::default());
                    cert_meta.received_count = cert_meta.received_count.saturating_add(1);
                    cert_meta.received_count
                });

                // Write StorageCertsRemovableOn and StorageCertsByIssuer
                let cert_value = CertValue {
                    chainable_on: block_number + T::RenewablePeriod::get(),
                    removable_on: block_number + T::ValidityPeriod::get(),
                    phantom: PhantomData,
                };
                <StorageCertsRemovableOn<T, I>>::append(cert_value.removable_on, (issuer, receiver));
                <StorageCertsByIssuer<T, I>>::insert(issuer, receiver, cert_value);

                if create {
                    // Write StorageCertsByReceiver
                    <StorageCertsByReceiver<T, I>>::mutate_exists(receiver, |issuers_opt| {
                        let issuers = issuers_opt.get_or_insert(vec![]);
                        if let Err(index) = issuers.binary_search(&issuer) {
                            issuers.insert(index, issuer);
                        }
                    });
                    Self::deposit_event(RawEvent::NewCert(issuer, issuer_issued_count, receiver, receiver_received_count));
                    T::OnNewcert::on_new_cert(issuer, issuer_issued_count, receiver, receiver_received_count);
                } else {
                    Self::deposit_event(RawEvent::RenewedCert(issuer, receiver));
                }
            }
            #[weight = 0]
            pub fn del_cert(origin, issuer: T::IdtyIndex, receiver: T::IdtyIndex) {
                T::DelCertOrigin::ensure_origin((origin, issuer, receiver))?;
                Self::remove_cert_inner(issuer, receiver, None);
            }
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config<I>, I: Instance> Module<T, I> {
        pub fn is_idty_allowed_to_create_cert(idty_index: T::IdtyIndex) -> bool {
            if let Ok(cert_meta) = <StorageIdtyCertMeta<T, I>>::try_get(idty_index) {
                cert_meta.next_issuable_on <= frame_system::pallet::Pallet::<T>::block_number()
                    && cert_meta.issued_count < T::MaxByIssuer::get()
            } else {
                true
            }
        }
        pub fn on_idty_removed(idty_index: T::IdtyIndex) -> Weight {
            let mut total_weight: Weight = 0;
            if let Ok(issuers) = <StorageCertsByReceiver<T, I>>::try_get(idty_index) {
                for issuer in issuers {
                    total_weight += Self::remove_cert_inner(issuer, idty_index, None);
                }
            }
            total_weight
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: Instance> Module<T, I> {
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
                Self::deposit_event(RawEvent::RemovedCert(
                    issuer,
                    issuer_issued_count,
                    receiver,
                    receiver_received_count,
                    block_number_opt.is_some(),
                ));
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
