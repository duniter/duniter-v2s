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
#![allow(clippy::type_complexity)]

pub mod traits;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/*#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

pub use pallet::*;
pub use types::*;

use crate::traits::*;
use codec::Codec;
use frame_support::dispatch::Weight;
use sp_runtime::traits::{AtLeast32BitUnsigned, One, Saturating, Zero};
use sp_std::fmt::Debug;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::StorageVersion;
    use frame_system::pallet_prelude::*;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        /// Period during which the owner can confirm the new identity.
        type ConfirmPeriod: Get<Self::BlockNumber>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Management of the authorizations of the different calls. (The default implementation only allows root)
        type EnsureIdtyCallAllowed: EnsureIdtyCallAllowed<Self>;
        /// Minimum duration between the creation of 2 identities by the same creator
        type IdtyCreationPeriod: Get<Self::BlockNumber>;
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
        /// Handle logic to validate an identity name
        type IdtyNameValidator: IdtyNameValidator;
        /// Origin allowed to validate identity
        type IdtyValidationOrigin: EnsureOrigin<Self::Origin>;
        ///
        type IsMember: sp_runtime::traits::IsMember<Self::IdtyIndex>;
        /// On identity confirmed by it's owner
        type OnIdtyChange: OnIdtyChange<Self>;
        #[pallet::constant]
        /// Maximum period with disabled status, after this period, the identity is permanently
        /// deleted
        type MaxDisabledPeriod: Get<Self::BlockNumber>;
    }

    // GENESIS STUFF //

    #[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq)]
    pub struct GenesisIdty<T: Config> {
        pub index: T::IdtyIndex,
        pub name: IdtyName,
        pub value: IdtyValue<T::BlockNumber, T::AccountId>,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub identities: Vec<GenesisIdty<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                identities: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let mut names = sp_std::collections::btree_set::BTreeSet::new();
            for idty in &self.identities {
                assert!(
                    !names.contains(&idty.name),
                    "Idty name {:?} is present twice",
                    &idty.name
                );
                assert!(idty.value.removable_on == T::BlockNumber::zero());
                names.insert(idty.name.clone());
            }

            let mut identities = self.identities.clone();
            identities.sort_unstable_by(|a, b| a.index.cmp(&b.index));

            for idty in identities.into_iter() {
                let idty_index = Pallet::<T>::get_next_idty_index();
                if idty.value.removable_on > T::BlockNumber::zero() {
                    <IdentitiesRemovableOn<T>>::append(
                        idty.value.removable_on,
                        (idty_index, idty.value.status),
                    )
                }
                <Identities<T>>::insert(idty_index, idty.value.clone());
                IdentitiesNames::<T>::insert(idty.name.clone(), ());
                IdentityIndexOf::<T>::insert(idty.value.owner_key, idty_index);
            }
        }
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn identity)]
    pub type Identities<T: Config> = CountedStorageMap<
        _,
        Twox64Concat,
        T::IdtyIndex,
        IdtyValue<T::BlockNumber, T::AccountId>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn identity_index_of)]
    pub type IdentityIndexOf<T: Config> =
        StorageMap<_, Blake2_128, T::AccountId, T::IdtyIndex, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn identity_by_did)]
    pub type IdentitiesNames<T: Config> = StorageMap<_, Blake2_128, IdtyName, (), OptionQuery>;

    #[pallet::storage]
    pub(super) type NextIdtyIndex<T: Config> = StorageValue<_, T::IdtyIndex, ValueQuery>;

    /// Identities by removed block
    #[pallet::storage]
    #[pallet::getter(fn removable_on)]
    pub type IdentitiesRemovableOn<T: Config> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<(T::IdtyIndex, IdtyStatus)>, ValueQuery>;

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if n > T::BlockNumber::zero() {
                Self::prune_identities(n)
            } else {
                0
            }
        }
    }

    // EVENTS //

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new identity has been created
        /// [idty_index, owner_key]
        IdtyCreated {
            idty_index: T::IdtyIndex,
            owner_key: T::AccountId,
        },
        /// An identity has been confirmed by it's owner
        /// [idty_index, owner_key, name]
        IdtyConfirmed {
            idty_index: T::IdtyIndex,
            owner_key: T::AccountId,
            name: IdtyName,
        },
        /// An identity has been validated
        /// [idty_index]
        IdtyValidated { idty_index: T::IdtyIndex },
        /// An identity has been removed
        /// [idty_index]
        IdtyRemoved { idty_index: T::IdtyIndex },
    }

    // CALLS //

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_identity(
            origin: OriginFor<T>,
            owner_key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;

            let creator =
                IdentityIndexOf::<T>::try_get(&who).map_err(|_| Error::<T>::IdtyIndexNotFound)?;
            let creator_idty_val =
                Identities::<T>::try_get(&creator).map_err(|_| Error::<T>::IdtyNotFound)?;

            if IdentityIndexOf::<T>::contains_key(&owner_key) {
                return Err(Error::<T>::IdtyAlreadyCreated.into());
            }

            if !T::EnsureIdtyCallAllowed::can_create_identity(creator) {
                return Err(Error::<T>::CreatorNotAllowedToCreateIdty.into());
            }

            let block_number = frame_system::pallet::Pallet::<T>::block_number();

            if creator_idty_val.next_creatable_identity_on > block_number {
                return Err(Error::<T>::NotRespectIdtyCreationPeriod.into());
            }

            // Apply phase //
            frame_system::Pallet::<T>::inc_sufficients(&owner_key);
            <Identities<T>>::mutate_exists(creator, |idty_val_opt| {
                if let Some(ref mut idty_val) = idty_val_opt {
                    idty_val.next_creatable_identity_on =
                        block_number + T::IdtyCreationPeriod::get();
                }
            });

            let removable_on = block_number + T::ConfirmPeriod::get();

            let idty_index = Self::get_next_idty_index();
            <Identities<T>>::insert(
                idty_index,
                IdtyValue {
                    next_creatable_identity_on: T::BlockNumber::zero(),
                    owner_key: owner_key.clone(),
                    removable_on,
                    status: IdtyStatus::Created,
                },
            );
            IdentitiesRemovableOn::<T>::append(removable_on, (idty_index, IdtyStatus::Created));
            IdentityIndexOf::<T>::insert(owner_key.clone(), idty_index);
            Self::deposit_event(Event::IdtyCreated {
                idty_index,
                owner_key,
            });
            T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Created { creator });
            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn confirm_identity(
            origin: OriginFor<T>,
            idty_name: IdtyName,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;

            let idty_index =
                IdentityIndexOf::<T>::try_get(&who).map_err(|_| Error::<T>::IdtyIndexNotFound)?;

            let mut idty_value =
                Identities::<T>::try_get(idty_index).map_err(|_| Error::<T>::IdtyNotFound)?;

            if idty_value.status != IdtyStatus::Created {
                return Err(Error::<T>::IdtyAlreadyConfirmed.into());
            }
            if !T::IdtyNameValidator::validate(&idty_name) {
                return Err(Error::<T>::IdtyNameInvalid.into());
            }
            if <IdentitiesNames<T>>::contains_key(&idty_name) {
                return Err(Error::<T>::IdtyNameAlreadyExist.into());
            }
            if !T::EnsureIdtyCallAllowed::can_confirm_identity(idty_index, who.clone()) {
                return Err(Error::<T>::NotAllowedToConfirmIdty.into());
            }

            // Apply phase //
            idty_value.status = IdtyStatus::ConfirmedByOwner;

            <Identities<T>>::insert(idty_index, idty_value);
            <IdentitiesNames<T>>::insert(idty_name.clone(), ());
            Self::deposit_event(Event::IdtyConfirmed {
                idty_index,
                owner_key: who,
                name: idty_name,
            });
            T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Confirmed);
            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn validate_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            T::IdtyValidationOrigin::ensure_origin(origin)?;

            let mut idty_value =
                Identities::<T>::try_get(idty_index).map_err(|_| Error::<T>::IdtyNotFound)?;

            match idty_value.status {
                IdtyStatus::Created => return Err(Error::<T>::IdtyNotConfirmedByOwner.into()),
                IdtyStatus::ConfirmedByOwner => {
                    if !T::EnsureIdtyCallAllowed::can_validate_identity(idty_index) {
                        return Err(Error::<T>::NotAllowedToValidateIdty.into());
                    }
                }
                IdtyStatus::Validated => return Err(Error::<T>::IdtyAlreadyValidated.into()),
            }

            // Apply phase //
            idty_value.removable_on = T::BlockNumber::zero();
            idty_value.status = IdtyStatus::Validated;

            <Identities<T>>::insert(idty_index, idty_value);
            Self::deposit_event(Event::IdtyValidated { idty_index });
            T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Validated);

            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn remove_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            idty_name: Option<IdtyName>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Self::do_remove_identity(idty_index);
            if let Some(idty_name) = idty_name {
                <IdentitiesNames<T>>::remove(idty_name);
            }

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn prune_item_identities_names(
            origin: OriginFor<T>,
            names: Vec<IdtyName>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            for name in names {
                <IdentitiesNames<T>>::remove(name);
            }

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn prune_item_identity_index_of(
            origin: OriginFor<T>,
            accounts_ids: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            accounts_ids
                .into_iter()
                .filter(|account_id| {
                    if let Ok(idty_index) = IdentityIndexOf::<T>::try_get(&account_id) {
                        !Identities::<T>::contains_key(&idty_index)
                    } else {
                        false
                    }
                })
                .for_each(IdentityIndexOf::<T>::remove);

            Ok(().into())
        }
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Creator not allowed to create identities
        CreatorNotAllowedToCreateIdty,
        /// Identity already confirmed
        IdtyAlreadyConfirmed,
        /// Identity already created
        IdtyAlreadyCreated,
        /// Identity already validated
        IdtyAlreadyValidated,
        /// You are not allowed to create a new identity now
        IdtyCreationNotAllowed,
        /// Identity index not found
        IdtyIndexNotFound,
        /// Identity name already exist
        IdtyNameAlreadyExist,
        /// Idty name invalid
        IdtyNameInvalid,
        /// Identity not confirmed by owner
        IdtyNotConfirmedByOwner,
        /// Identity not found
        IdtyNotFound,
        /// Idty not member
        IdtyNotMember,
        /// Identity not validated
        IdtyNotValidated,
        /// Identity not yet renewable
        IdtyNotYetRenewable,
        /// Not allowed to confirm identity
        NotAllowedToConfirmIdty,
        /// Not allowed to validate identity
        NotAllowedToValidateIdty,
        /// Not same identity name
        NotSameIdtyName,
        /// Right already added
        RightAlreadyAdded,
        /// Right not exist
        RightNotExist,
        /// Not respect IdtyCreationPeriod
        NotRespectIdtyCreationPeriod,
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn identities_count() -> u32 {
            Identities::<T>::count()
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub(super) fn do_remove_identity(idty_index: T::IdtyIndex) -> Weight {
            if let Some(idty_val) = Identities::<T>::take(idty_index) {
                frame_system::Pallet::<T>::dec_sufficients(&idty_val.owner_key);
            }
            Self::deposit_event(Event::IdtyRemoved { idty_index });
            T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Removed);
            0
        }
        fn get_next_idty_index() -> T::IdtyIndex {
            if let Ok(next_index) = <NextIdtyIndex<T>>::try_get() {
                <NextIdtyIndex<T>>::put(next_index.saturating_add(T::IdtyIndex::one()));
                next_index
            } else {
                <NextIdtyIndex<T>>::put(T::IdtyIndex::one() + T::IdtyIndex::one());
                T::IdtyIndex::one()
            }
        }
        fn prune_identities(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0;

            for (idty_index, idty_status) in IdentitiesRemovableOn::<T>::take(block_number) {
                if let Ok(idty_val) = <Identities<T>>::try_get(idty_index) {
                    if idty_val.removable_on == block_number && idty_val.status == idty_status {
                        total_weight += Self::do_remove_identity(idty_index)
                    }
                }
            }

            total_weight
        }
    }
}

impl<T: Config> sp_runtime::traits::Convert<T::AccountId, Option<T::IdtyIndex>> for Pallet<T> {
    fn convert(account_id: T::AccountId) -> Option<T::IdtyIndex> {
        Self::identity_index_of(account_id)
    }
}
