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
        ///  Identity custom data
        type IdtyData: Parameter + Member + MaybeSerializeDeserialize + Debug + Default;
        ///  Identity custom data provider
        type IdtyDataProvider: ProvideIdtyData<Self>;
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

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub identities: Vec<IdtyValue<T::AccountId, T::BlockNumber, T::IdtyData>>,
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
            for idty_value in &self.identities {
                assert!(
                    !names.contains(&idty_value.name),
                    "Idty name {:?} is present twice",
                    &idty_value.name
                );
                assert!(idty_value.removable_on == T::BlockNumber::zero());
                names.insert(idty_value.name.clone());
            }

            // We need to sort identities to ensure determinisctic result
            let mut identities = self.identities.clone();
            identities.sort_by(|idty_val_1, idty_val_2| idty_val_1.name.cmp(&idty_val_2.name));

            <IdentitiesCount<T>>::put(self.identities.len() as u64);
            for idty_value in &identities {
                let idty_index = Pallet::<T>::get_next_idty_index();
                if idty_value.removable_on > T::BlockNumber::zero() {
                    <IdentitiesRemovableOn<T>>::append(
                        idty_value.removable_on,
                        (idty_index, idty_value.status),
                    )
                }
                <Identities<T>>::insert(idty_index, idty_value);
            }
        }
    }

    // STORAGE //

    /// Identities
    #[pallet::storage]
    #[pallet::getter(fn identity)]
    pub type Identities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::IdtyIndex,
        IdtyValue<T::AccountId, T::BlockNumber, T::IdtyData>,
        OptionQuery,
    >;

    /// IdentitiesByDid
    #[pallet::storage]
    #[pallet::getter(fn identity_by_did)]
    pub type IdentitiesByDid<T: Config> =
        StorageMap<_, Blake2_128Concat, IdtyName, T::IdtyIndex, ValueQuery>;

    #[pallet::storage]
    pub(super) type NextIdtyIndex<T: Config> = StorageValue<_, T::IdtyIndex, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn identities_count)]
    pub(super) type IdentitiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Identities by removed block
    #[pallet::storage]
    #[pallet::getter(fn removable_on)]
    pub type IdentitiesRemovableOn<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::BlockNumber,
        Vec<(T::IdtyIndex, IdtyStatus)>,
        ValueQuery,
    >;

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
        /// [idty, owner_key]
        IdtyCreated(IdtyName, T::AccountId),
        /// An identity has been confirmed by it's owner
        /// [idty]
        IdtyConfirmed(IdtyName),
        /// An identity has been validated
        /// [idty]
        IdtyValidated(IdtyName),
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
            creator: T::IdtyIndex,
            idty_name: IdtyName,
            owner_key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;

            let creator_idty_val =
                Identities::<T>::try_get(&creator).map_err(|_| Error::<T>::CreatorNotExist)?;

            if who != creator_idty_val.owner_key {
                return Err(Error::<T>::RequireToBeOwner.into());
            }

            if !T::EnsureIdtyCallAllowed::can_create_identity(creator) {
                return Err(Error::<T>::CreatorNotAllowedToCreateIdty.into());
            }

            let block_number = frame_system::pallet::Pallet::<T>::block_number();

            if creator_idty_val.next_creatable_identity_on > block_number {
                return Err(Error::<T>::NotRespectIdtyCreationPeriod.into());
            }

            if !T::IdtyNameValidator::validate(&idty_name) {
                return Err(Error::<T>::IdtyNameInvalid.into());
            }
            if <IdentitiesByDid<T>>::contains_key(&idty_name) {
                return Err(Error::<T>::IdtyNameAlreadyExist.into());
            }

            // Apply phase //

            <Identities<T>>::mutate_exists(creator, |idty_val_opt| {
                if let Some(ref mut idty_val) = idty_val_opt {
                    idty_val.next_creatable_identity_on =
                        block_number + T::IdtyCreationPeriod::get();
                }
            });

            let idty_data =
                T::IdtyDataProvider::provide_identity_data(creator, &idty_name, &owner_key);

            let removable_on = block_number + T::ConfirmPeriod::get();

            let idty_index = Self::get_next_idty_index();
            <Identities<T>>::insert(
                idty_index,
                IdtyValue {
                    name: idty_name.clone(),
                    next_creatable_identity_on: T::BlockNumber::zero(),
                    owner_key: owner_key.clone(),
                    removable_on,
                    status: IdtyStatus::Created,
                    data: idty_data,
                },
            );
            <IdentitiesByDid<T>>::insert(idty_name.clone(), idty_index);
            IdentitiesRemovableOn::<T>::append(removable_on, (idty_index, IdtyStatus::Created));
            Self::inc_identities_counter();
            Self::deposit_event(Event::IdtyCreated(idty_name, owner_key));
            T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Created { creator });
            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn confirm_identity(
            origin: OriginFor<T>,
            idty_name: IdtyName,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                if who == idty_value.owner_key {
                    if idty_value.status != IdtyStatus::Created {
                        return Err(Error::<T>::IdtyAlreadyConfirmed.into());
                    }
                    if idty_value.name != idty_name {
                        return Err(Error::<T>::NotSameIdtyName.into());
                    }
                    if !T::EnsureIdtyCallAllowed::can_confirm_identity(idty_index) {
                        return Err(Error::<T>::NotAllowedToConfirmIdty.into());
                    }

                    idty_value.status = IdtyStatus::ConfirmedByOwner;

                    <Identities<T>>::insert(idty_index, idty_value);
                    Self::deposit_event(Event::IdtyConfirmed(idty_name));
                    T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Confirmed);
                    Ok(().into())
                } else {
                    Err(Error::<T>::RequireToBeOwner.into())
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
        #[pallet::weight(0)]
        pub fn validate_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            T::IdtyValidationOrigin::ensure_origin(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                match idty_value.status {
                    IdtyStatus::Created => Err(Error::<T>::IdtyNotConfirmedByOwner.into()),
                    IdtyStatus::ConfirmedByOwner | IdtyStatus::Disabled => {
                        if !T::EnsureIdtyCallAllowed::can_validate_identity(idty_index) {
                            return Err(Error::<T>::NotAllowedToValidateIdty.into());
                        }

                        idty_value.removable_on = T::BlockNumber::zero();
                        idty_value.status = IdtyStatus::Validated;
                        let name = idty_value.name.clone();

                        <Identities<T>>::insert(idty_index, idty_value);
                        Self::deposit_event(Event::IdtyValidated(name));
                        T::OnIdtyChange::on_idty_change(idty_index, IdtyEvent::Validated);
                        Ok(().into())
                    }
                    IdtyStatus::Validated => Err(Error::<T>::IdtyAlreadyValidated.into()),
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
        #[pallet::weight(0)]
        pub fn disable_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            // Verify phase
            ensure_root(origin)?;

            if !Identities::<T>::contains_key(idty_index) {
                return Err(Error::<T>::IdtyNotFound.into());
            }

            // Apply phase
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let removable_on = block_number + T::MaxDisabledPeriod::get();
            Identities::<T>::mutate_exists(idty_index, |idty_value_opt| {
                if let Some(idty_value) = idty_value_opt {
                    idty_value.status = IdtyStatus::Disabled;
                    idty_value.removable_on = removable_on;
                }
            });
            <IdentitiesRemovableOn<T>>::append(removable_on, (idty_index, IdtyStatus::Disabled));
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn remove_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Self::do_remove_identity(idty_index);

            Ok(().into())
        }
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Creator not exist
        CreatorNotExist,
        /// Creator not allowed to create identities
        CreatorNotAllowedToCreateIdty,
        /// Identity already confirmed
        IdtyAlreadyConfirmed,
        /// Identity already validated
        IdtyAlreadyValidated,
        /// You are not allowed to create a new identity now
        IdtyCreationNotAllowed,
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
        /// This operation requires to be the owner of the identity
        RequireToBeOwner,
        /// Right already added
        RightAlreadyAdded,
        /// Right not exist
        RightNotExist,
        /// Not respect IdtyCreationPeriod
        NotRespectIdtyCreationPeriod,
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn set_idty_data(idty_index: T::IdtyIndex, idty_data: T::IdtyData) {
            Identities::<T>::mutate_exists(idty_index, |idty_val_opt| {
                if let Some(ref mut idty_val) = idty_val_opt {
                    idty_val.data = idty_data;
                }
            });
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        fn dec_identities_counter() {
            if let Ok(counter) = <IdentitiesCount<T>>::try_get() {
                <IdentitiesCount<T>>::put(counter.saturating_sub(1));
            } else {
                panic!("storage corrupted")
            }
        }
        pub(super) fn do_remove_identity(idty_index: T::IdtyIndex) -> Weight {
            if let Some(idty_val) = <Identities<T>>::take(idty_index) {
                <IdentitiesByDid<T>>::remove(idty_val.name);
            }
            Self::dec_identities_counter();
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
        fn inc_identities_counter() {
            if let Ok(counter) = <IdentitiesCount<T>>::try_get() {
                <IdentitiesCount<T>>::put(counter.saturating_add(1));
            } else {
                <IdentitiesCount<T>>::put(1);
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
