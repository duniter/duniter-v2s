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

pub use pallet::*;

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
        /// Duration after which a certification is renewable
        type ChainabilityPeriod: Get<Self::BlockNumber>;
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
        type MaxByIssuer: Get<u32>;
        /// Minimum duration between two certifications issued by the same issuer
        type SignPeriod: Get<Self::BlockNumber>;
        /// Duration of validity of a certification
        type ValidityPeriod: Get<Self::BlockNumber>;
    }

    frame_support::decl_event! {
        pub enum Event<T, I=DefaultInstance> where
            <T as frame_system::Config>::Hash,
            <T as frame_system::Config>::AccountId,
        {
            /// A motion (given hash) has been proposed (by given account) with a threshold (given
            /// `MemberCount`).
            /// \[account, proposal_hash\]
            Proposed(AccountId, Hash),
        }
    }

    frame_support::decl_error! {
        pub enum Error for Module<T: Config<I>, I: Instance> {
            /// Account is not a member
            NotMember,
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
        receiver: T::IdtyIndex,
        chainable_on: T::BlockNumber,
        removable_on: T::BlockNumber,
    }

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
    pub struct CertsByIssuer<T: Config<I>, I: Instance> {
        certs: Vec<CertValue<T, I>>,
        next_issuable_on: T::BlockNumber,
    }

    frame_support::decl_storage! {
        trait Store for Module<T: Config<I>, I: Instance=DefaultInstance> as Certification {
            /// Storage version of the pallet.
            StorageVersion get(fn storage_version): Releases;
            /// Certifications by issuer
            pub StorageCertsByIssuer get(fn certs_by_issuer):
            map hasher(twox_64_concat) T::IdtyIndex => CertsByIssuer<T, I> = CertsByIssuer {
                certs: vec![],
                next_issuable_on: T::BlockNumber::zero(),
            };
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
                let mut certs_by_receiver = BTreeMap::<T::IdtyIndex, Vec<T::IdtyIndex>>::new();
                for (issuer, receivers) in &config.certs_by_issuer {
                    assert!(!receivers.contains(issuer), "Identity cannot tcertify it-self.");
                    assert!(!receivers.len() <= T::MaxByIssuer::get() as usize, "Identity n°{:?} exceed MaxByIssuer.", issuer);
                    for receiver in receivers {
                        certs_by_receiver.entry(*receiver).or_default().push(*issuer);
                    }
                }

                <StorageVersion<I>>::put(Releases::V1_0_0);
                let mut all_couples = Vec::new();
                for (issuer, receivers) in &config.certs_by_issuer {
                    let mut certs = Vec::with_capacity(receivers.len());
                    for receiver in receivers {
                        all_couples.push((*issuer, *receiver));
                        certs.push(CertValue {
                            receiver: *receiver,
                            chainable_on: T::ChainabilityPeriod::get(),
                            removable_on: T::ValidityPeriod::get(),
                        });
                        let received_certs = certs_by_receiver.remove(receiver).unwrap_or_default();
                        <StorageCertsByReceiver<T, I>>::insert(receiver, received_certs);

                    }
                    <StorageCertsByIssuer<T, I>>::insert(issuer, CertsByIssuer {
                        certs,
                        next_issuable_on: T::SignPeriod::get(),
                    });
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
            if let Ok(mut certs_by_issuer) = <StorageCertsByIssuer<T, I>>::try_get(issuer) {
                if let Ok(index) = certs_by_issuer.certs.binary_search_by(
                    |CertValue {
                         receiver: receiver_,
                         ..
                     }| receiver.cmp(&receiver_),
                ) {
                    if let Some(cert_val) = certs_by_issuer.certs.get(index) {
                        if Some(cert_val.removable_on) == block_number_opt
                            || block_number_opt.is_none()
                        {
                            certs_by_issuer.certs.remove(index);
                            <StorageCertsByIssuer<T, I>>::insert(issuer, certs_by_issuer);
                            if let Ok(mut certs_by_receiver) =
                                <StorageCertsByReceiver<T, I>>::try_get(receiver)
                            {
                                if let Ok(index) = certs_by_receiver
                                    .binary_search_by(|issuer_| issuer.cmp(&issuer_))
                                {
                                    certs_by_receiver.remove(index);
                                }
                            }
                        }
                    }
                }
            }
            0
        }
    }
}
