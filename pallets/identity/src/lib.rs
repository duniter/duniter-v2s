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

#[cfg(test)]
mod tests;

/*#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

pub use pallet::*;

use crate::traits::*;
use codec::Codec;
use frame_support::dispatch::Weight;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::{AtLeast32BitUnsigned, One, Saturating, Zero};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::fmt::Debug;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        /// Period during which the owner can confirm the new identity.
        type ConfirmPeriod: Get<Self::BlockNumber>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Origin allowed to add a right to an identity
        type AddRightOrigin: EnsureOrigin<Self::Origin>;
        /// Origin allowed to delete a right to an identity
        type DelRightOrigin: EnsureOrigin<Self::Origin>;
        /// Management of the authorizations of the different calls. (The default implementation only allows root)
        type EnsureIdtyCallAllowed: EnsureIdtyCallAllowed<Self>;
        ///  Identity custom data
        type IdtyData: IdtyData;
        /// Identity decentralized identifier
        type IdtyDid: IdtyDid;
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
        /// Origin allowed to validate identity
        type IdtyValidationOrigin: EnsureOrigin<Self::Origin>;
        /// Rights that an identity can have
        type IdtyRight: IdtyRight;
        /// On identity confirmed by it's owner
        type OnIdtyConfirmed: OnIdtyConfirmed<Self>;
        /// On identity removed
        type OnIdtyRemoved: OnIdtyRemoved<Self>;
        /// On identity validated
        type OnIdtyValidated: OnIdtyValidated<Self>;
        /// On right key change
        type OnRightKeyChange: OnRightKeyChange<Self>;
        #[pallet::constant]
        /// Maximum period of inactivity, after this period, the identity is permanently deleted
        type MaxInactivityPeriod: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Period after which a non-validated identity is deleted
        type ValidationPeriod: Get<Self::BlockNumber>;
    }

    // STORAGE //

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

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

    #[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub enum IdtyStatus {
        Created,
        ConfirmedByOwner,
        Validated,
    }
    impl Default for IdtyStatus {
        fn default() -> Self {
            IdtyStatus::Created
        }
    }

    #[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq)]
    pub struct IdtyValue<T: Config> {
        pub did: T::IdtyDid,
        pub owner_key: T::AccountId,
        pub removable_on: Option<T::BlockNumber>,
        pub rights: Vec<(T::IdtyRight, Option<T::AccountId>)>,
        pub status: IdtyStatus,
        pub data: T::IdtyData,
    }
    impl<T: Config> IdtyValue<T> {
        pub fn new_valid(
            did: T::IdtyDid,
            owner_key: T::AccountId,
            rights: Vec<T::IdtyRight>,
        ) -> Self {
            Self {
                did,
                owner_key,
                removable_on: None,
                rights: rights.into_iter().map(|right| (right, None)).collect(),
                status: IdtyStatus::Validated,
                data: T::IdtyData::default(),
            }
        }
    }
    impl<T: Config> Default for IdtyValue<T> {
        fn default() -> Self {
            Self {
                did: Default::default(),
                owner_key: Default::default(),
                removable_on: None,
                rights: Default::default(),
                status: Default::default(),
                data: Default::default(),
            }
        }
    }
    impl<T: Config> IdtyValue<T> {
        pub fn get_right_key(&self, right: T::IdtyRight) -> Option<T::AccountId> {
            if let Ok(index) = self
                .rights
                .binary_search_by(|(right_, _)| right.cmp(right_))
            {
                if self.rights[index].1.is_some() {
                    self.rights[index].1.clone()
                } else if right.allow_owner_key() {
                    Some(self.owner_key.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// Storage version of the pallet.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    /// Identities
    #[pallet::storage]
    #[pallet::getter(fn identity)]
    pub type Identities<T: Config> =
        StorageMap<_, Blake2_128Concat, T::IdtyIndex, IdtyValue<T>, ValueQuery>;

    /// IdentitiesByDid
    #[pallet::storage]
    #[pallet::getter(fn identity_by_did)]
    pub type IdentitiesByDid<T: Config> =
        StorageMap<_, Blake2_128Concat, T::IdtyDid, T::IdtyIndex, ValueQuery>;

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

    // GENESIS //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub identities: Vec<IdtyValue<T>>,
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
            let mut dids = BTreeSet::new();
            for idty_value in &self.identities {
                assert!(
                    !dids.contains(&idty_value.did),
                    "Did {:?} is present twice",
                    idty_value.did
                );
                if idty_value.status == IdtyStatus::Validated {
                    if idty_value.rights.is_empty() {
                        assert!(idty_value.removable_on.is_some());
                    } else {
                        assert!(idty_value.removable_on.is_none());
                    }
                } else {
                    assert!(idty_value.removable_on.is_some());
                    assert!(idty_value.rights.is_empty())
                }
                dids.insert(idty_value.did);
            }

            // We need to sort identities to ensure determinisctic result
            let mut identities = self.identities.clone();
            identities.sort_by(|idty_val_1, idty_val_2| idty_val_1.did.cmp(&idty_val_2.did));

            <StorageVersion<T>>::put(Releases::V1_0_0);
            <IdentitiesCount<T>>::put(self.identities.len() as u64);
            for idty_value in &identities {
                let idty_index = Pallet::<T>::get_next_idty_index();
                if let Some(removable_on) = idty_value.removable_on {
                    <IdentitiesRemovableOn<T>>::append(
                        removable_on,
                        (idty_index, idty_value.status),
                    )
                }
                <Identities<T>>::insert(idty_index, idty_value);
            }
        }
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            Self::prune_identities(n)
        }
    }

    // EVENTS //

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(
        T::IdtyDid = "IdtyDid",
        T::IdtyRight = "IdtyRight",
        T::AccountId = "AccountId"
    )]
    pub enum Event<T: Config> {
        /// A new identity has been created
        /// [idty, owner_key]
        IdtyCreated(T::IdtyDid, T::AccountId),
        /// An identity has been confirmed by it's owner
        /// [idty]
        IdtyConfirmed(T::IdtyDid),
        /// An identity has been validated
        /// [idty]
        IdtyValidated(T::IdtyDid),
        /// An identity was declared dead
        /// [idty]
        IdtyDead(T::IdtyDid),
        /// An identity has acquired a new right
        /// [idty, right]
        IdtyAcquireRight(T::IdtyDid, T::IdtyRight),
        /// An identity lost a right
        /// [idty, righ]
        IdtyLostRight(T::IdtyDid, T::IdtyRight),
        /// An identity has modified a subkey associated with a right
        /// [idty_did, right, old_subkey_opt, new_subkey_opt]
        IdtySetRightSubKey(
            T::IdtyDid,
            T::IdtyRight,
            Option<T::AccountId>,
            Option<T::AccountId>,
        ),
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
            idty_did: T::IdtyDid,
            owner_key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            if !T::EnsureIdtyCallAllowed::create_identity(origin, creator, &idty_did, &owner_key) {
                return Err(Error::<T>::IdtyCreationNotAllowed.into());
            }
            if <IdentitiesByDid<T>>::contains_key(&idty_did) {
                return Err(Error::<T>::IdtyAlreadyExist.into());
            }

            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let removable_on = block_number + T::ConfirmPeriod::get();

            let idty_index = Self::get_next_idty_index();
            <Identities<T>>::insert(
                idty_index,
                IdtyValue {
                    did: idty_did,
                    owner_key: owner_key.clone(),
                    removable_on: Some(removable_on),
                    ..Default::default()
                },
            );
            <IdentitiesByDid<T>>::insert(idty_did, idty_index);
            IdentitiesRemovableOn::<T>::append(removable_on, (idty_index, IdtyStatus::Created));
            Self::inc_identities_counter();
            Self::deposit_event(Event::IdtyCreated(idty_did, owner_key));
            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn confirm_identity(
            origin: OriginFor<T>,
            idty_did: T::IdtyDid,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                if who == idty_value.owner_key {
                    if idty_value.status != IdtyStatus::Created {
                        return Err(Error::<T>::IdtyAlreadyConfirmed.into());
                    }

                    let block_number = frame_system::pallet::Pallet::<T>::block_number();
                    let removable_on = block_number + T::ValidationPeriod::get();
                    idty_value.removable_on = Some(removable_on);
                    idty_value.status = IdtyStatus::ConfirmedByOwner;
                    let owner_key = idty_value.owner_key.clone();

                    <Identities<T>>::insert(idty_index, idty_value);
                    IdentitiesRemovableOn::<T>::append(
                        removable_on,
                        (idty_index, IdtyStatus::ConfirmedByOwner),
                    );
                    Self::deposit_event(Event::IdtyConfirmed(idty_did));
                    T::OnIdtyConfirmed::on_idty_confirmed(idty_did, owner_key, removable_on);
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
                    IdtyStatus::ConfirmedByOwner => {
                        let block_number = frame_system::pallet::Pallet::<T>::block_number();
                        let removable_on = block_number + T::MaxInactivityPeriod::get();
                        idty_value.removable_on = Some(removable_on);
                        idty_value.status = IdtyStatus::Validated;
                        let owner_key = idty_value.owner_key.clone();
                        let did = idty_value.did;

                        <Identities<T>>::insert(idty_index, idty_value);
                        <IdentitiesRemovableOn<T>>::append(
                            removable_on,
                            (idty_index, IdtyStatus::Validated),
                        );
                        Self::deposit_event(Event::IdtyValidated(did));
                        T::OnIdtyValidated::on_idty_validated(idty_index, owner_key)?;
                        Ok(().into())
                    }
                    IdtyStatus::Validated => Err(Error::<T>::IdtyAlreadyValidated.into()),
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
        #[pallet::weight(0)]
        pub fn validate_identity_and_add_rights(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            rights: Vec<T::IdtyRight>,
        ) -> DispatchResultWithPostInfo {
            T::IdtyValidationOrigin::ensure_origin(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                match idty_value.status {
                    IdtyStatus::Created => Err(Error::<T>::IdtyNotConfirmedByOwner.into()),
                    IdtyStatus::ConfirmedByOwner => {
                        idty_value.removable_on = None;
                        idty_value.rights = rights.iter().map(|right| (*right, None)).collect();
                        idty_value.status = IdtyStatus::Validated;
                        let owner_key = idty_value.owner_key.clone();
                        let did = idty_value.did;

                        <Identities<T>>::insert(idty_index, idty_value);
                        Self::deposit_event(Event::IdtyValidated(did));
                        for right in rights {
                            Self::deposit_event(Event::IdtyAcquireRight(did, right));
                        }
                        T::OnIdtyValidated::on_idty_validated(idty_index, owner_key)?;
                        Ok(().into())
                    }
                    IdtyStatus::Validated => Err(Error::<T>::IdtyAlreadyValidated.into()),
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
        #[pallet::weight(0)]
        pub fn add_right(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            right: T::IdtyRight,
        ) -> DispatchResultWithPostInfo {
            T::AddRightOrigin::ensure_origin(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                if idty_value.status != IdtyStatus::Validated {
                    return Err(Error::<T>::IdtyNotValidated.into());
                }

                if let Err(index) = idty_value
                    .rights
                    .binary_search_by(|(right_, _)| right.cmp(right_))
                {
                    let did = idty_value.did;
                    let new_key = if right.allow_owner_key() {
                        Some(idty_value.owner_key.clone())
                    } else {
                        None
                    };

                    idty_value.removable_on = None;
                    idty_value.rights.insert(index, (right, None));
                    <Identities<T>>::insert(idty_index, idty_value);
                    Self::deposit_event(Event::<T>::IdtyAcquireRight(did, right));
                    if new_key.is_some() {
                        T::OnRightKeyChange::on_right_key_change(idty_index, right, None, new_key);
                    }
                    Ok(().into())
                } else {
                    Err(Error::<T>::RightAlreadyAdded.into())
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
        #[pallet::weight(0)]
        pub fn del_right(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            right: T::IdtyRight,
        ) -> DispatchResultWithPostInfo {
            T::DelRightOrigin::ensure_origin(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                if idty_value.status != IdtyStatus::Validated {
                    return Err(Error::<T>::IdtyNotValidated.into());
                }

                if let Ok(index) = idty_value
                    .rights
                    .binary_search_by(|(right_, _)| right.cmp(right_))
                {
                    let did = idty_value.did;
                    let old_key_opt = if let Some(ref subkey) = idty_value.rights[index].1 {
                        Some(subkey.clone())
                    } else if right.allow_owner_key() {
                        Some(idty_value.owner_key.clone())
                    } else {
                        None
                    };
                    idty_value.rights.remove(index);

                    if idty_value.rights.is_empty() {
                        let block_number = frame_system::pallet::Pallet::<T>::block_number();
                        let removable_on = block_number + T::MaxInactivityPeriod::get();
                        idty_value.removable_on = Some(removable_on);
                        <IdentitiesRemovableOn<T>>::append(
                            removable_on,
                            (idty_index, IdtyStatus::Validated),
                        );
                    }

                    <Identities<T>>::insert(idty_index, idty_value);
                    Self::deposit_event(Event::<T>::IdtyLostRight(did, right));
                    if old_key_opt.is_some() {
                        T::OnRightKeyChange::on_right_key_change(
                            idty_index,
                            right,
                            old_key_opt,
                            None,
                        );
                    }
                    Ok(().into())
                } else {
                    Err(Error::<T>::RightNotExist.into())
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
        #[pallet::weight(0)]
        pub fn set_right_subkey(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            right: T::IdtyRight,
            subkey_opt: Option<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if let Ok(mut idty_value) = <Identities<T>>::try_get(idty_index) {
                if who == idty_value.owner_key {
                    if idty_value.status != IdtyStatus::Validated {
                        return Err(Error::<T>::IdtyNotValidated.into());
                    }

                    if let Ok(index) = idty_value
                        .rights
                        .binary_search_by(|(right_, _)| right.cmp(right_))
                    {
                        let did = idty_value.did;
                        let old_subkey_opt = idty_value.rights[index].1.clone();
                        idty_value.rights[index].1 = subkey_opt.clone();
                        let new_key = if let Some(ref subkey) = subkey_opt {
                            Some(subkey.clone())
                        } else if right.allow_owner_key() {
                            Some(idty_value.owner_key.clone())
                        } else {
                            None
                        };

                        <Identities<T>>::insert(idty_index, idty_value);
                        Self::deposit_event(Event::<T>::IdtySetRightSubKey(
                            did,
                            right,
                            old_subkey_opt.clone(),
                            subkey_opt,
                        ));
                        T::OnRightKeyChange::on_right_key_change(
                            idty_index,
                            right,
                            old_subkey_opt,
                            new_key,
                        );
                        Ok(().into())
                    } else {
                        Err(Error::<T>::RightNotExist.into())
                    }
                } else {
                    Err(Error::<T>::RequireToBeOwner.into())
                }
            } else {
                Err(Error::<T>::IdtyNotFound.into())
            }
        }
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Identity already confirmed
        IdtyAlreadyConfirmed,
        /// Identity already exist
        IdtyAlreadyExist,
        /// Identity already validated
        IdtyAlreadyValidated,
        /// You are not allowed to create a new identity now
        IdtyCreationNotAllowed,
        /// Identity not confirmed by owner
        IdtyNotConfirmedByOwner,
        /// Identity not found
        IdtyNotFound,
        /// Identity not validated
        IdtyNotValidated,
        /// This operation requires to be the owner of the identity
        RequireToBeOwner,
        /// Right already added
        RightAlreadyAdded,
        /// Right not exist
        RightNotExist,
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        fn dec_identities_counter() {
            if let Ok(counter) = <IdentitiesCount<T>>::try_get() {
                <IdentitiesCount<T>>::put(counter.saturating_sub(1));
            } else {
                panic!("storage corrupted")
            }
        } //NextIdtyIndex
        fn get_next_idty_index() -> T::IdtyIndex {
            if let Ok(next_index) = <NextIdtyIndex<T>>::try_get() {
                <NextIdtyIndex<T>>::put(next_index.saturating_add(T::IdtyIndex::one()));
                next_index
            } else {
                <NextIdtyIndex<T>>::put(T::IdtyIndex::one());
                T::IdtyIndex::zero()
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

            use frame_support::storage::generator::StorageMap as _;
            if let Some(identities) = IdentitiesRemovableOn::<T>::from_query_to_optional_value(
                IdentitiesRemovableOn::<T>::take(block_number),
            ) {
                for (idty_index, idty_status) in identities {
                    if let Ok(idty_val) = <Identities<T>>::try_get(idty_index) {
                        if idty_val.removable_on == Some(block_number)
                            && idty_val.status == idty_status
                        {
                            let did = idty_val.did;
                            <Identities<T>>::remove(idty_index);
                            <IdentitiesByDid<T>>::remove(did);
                            Self::dec_identities_counter();
                            total_weight += T::OnIdtyRemoved::on_idty_removed(
                                idty_index,
                                did,
                                idty_val.owner_key,
                            );
                        }
                    }
                }
            }

            total_weight
        }
    }
}
