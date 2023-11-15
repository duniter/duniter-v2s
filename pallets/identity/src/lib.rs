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
#![allow(clippy::type_complexity)]

pub mod traits;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use crate::traits::*;
use codec::Codec;
use frame_support::dispatch::Weight;
use sp_runtime::traits::{AtLeast32BitUnsigned, IdentifyAccount, One, Saturating, Verify, Zero};
use sp_std::fmt::Debug;
use sp_std::prelude::*;

// icok = identity change owner key
pub const NEW_OWNER_KEY_PAYLOAD_PREFIX: [u8; 4] = [b'i', b'c', b'o', b'k'];
// revo = revocation
pub const REVOCATION_PAYLOAD_PREFIX: [u8; 4] = [b'r', b'e', b'v', b'o'];
// link = link (identity with account)
pub const LINK_IDTY_PAYLOAD_PREFIX: [u8; 4] = [b'l', b'i', b'n', b'k'];

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::StorageVersion;
    use frame_system::pallet_prelude::*;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        /// Period during which the owner can confirm the new identity.
        type ConfirmPeriod: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Minimum duration between two owner key changes
        type ChangeOwnerKeyPeriod: Get<Self::BlockNumber>;
        /// Management of the authorizations of the different calls.
        /// The default implementation allows everything.
        type CheckIdtyCallAllowed: CheckIdtyCallAllowed<Self>;
        #[pallet::constant]
        /// Minimum duration between the creation of 2 identities by the same creator
        type IdtyCreationPeriod: Get<Self::BlockNumber>;
        /// Custom data to store in each identity
        type IdtyData: Clone
            + Codec
            + Default
            + Eq
            + TypeInfo
            + MaybeSerializeDeserialize
            + MaxEncodedLen;
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
        /// custom type for account data
        type AccountLinker: LinkIdty<Self::AccountId, Self::IdtyIndex>;
        /// Handle logic to validate an identity name
        type IdtyNameValidator: IdtyNameValidator;
        /// Additional reasons for identity removal
        type IdtyRemovalOtherReason: Clone + Codec + Debug + Eq + TypeInfo;
        /// On identity confirmed by its owner
        type OnIdtyChange: OnIdtyChange<Self>;
        /// Signing key of a payload
        type Signer: IdentifyAccount<AccountId = Self::AccountId>;
        /// Signature of a payload
        type Signature: Parameter + Verify<Signer = Self::Signer>;
        /// Handle the logic that removes all identity consumers.
        /// "identity consumers" meaning all things that rely on the existence of the identity.
        type RemoveIdentityConsumers: RemoveIdentityConsumers<Self::IdtyIndex>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
        /// Type representing the a distance handler to prepare identity for benchmarking
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkSetupHandler: SetupBenchmark<Self::IdtyIndex, Self::AccountId>;
    }

    // GENESIS STUFF //

    #[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq)]
    pub struct GenesisIdty<T: Config> {
        pub index: T::IdtyIndex,
        pub name: IdtyName,
        pub value: IdtyValue<T::BlockNumber, T::AccountId, T::IdtyData>,
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
                names.insert(idty.name.clone());
            }

            let mut identities = self.identities.clone();
            identities.sort_unstable_by(|a, b| a.index.cmp(&b.index));

            for idty in identities.into_iter() {
                // if get_next_idty_index() is not called
                // then NextIdtyIndex is not incremented
                let _ = Pallet::<T>::get_next_idty_index();
                // use instead custom provided index
                let idty_index = idty.index;
                if idty.value.removable_on > T::BlockNumber::zero() {
                    <IdentitiesRemovableOn<T>>::append(
                        idty.value.removable_on,
                        (idty_index, idty.value.status),
                    )
                }
                <Identities<T>>::insert(idty_index, idty.value.clone());
                IdentitiesNames::<T>::insert(idty.name.clone(), idty_index);
                IdentityIndexOf::<T>::insert(&idty.value.owner_key, idty_index);
                frame_system::Pallet::<T>::inc_sufficients(&idty.value.owner_key);
                if let Some((old_owner_key, _last_change)) = idty.value.old_owner_key {
                    frame_system::Pallet::<T>::inc_sufficients(&old_owner_key);
                }
            }
        }
    }

    // STORAGE //

    /// maps identity index to identity value
    #[pallet::storage]
    #[pallet::getter(fn identity)]
    pub type Identities<T: Config> = CountedStorageMap<
        _,
        Twox64Concat,
        T::IdtyIndex,
        IdtyValue<T::BlockNumber, T::AccountId, T::IdtyData>,
        OptionQuery,
    >;

    /// maps account id to identity index
    #[pallet::storage]
    #[pallet::getter(fn identity_index_of)]
    pub type IdentityIndexOf<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::IdtyIndex, OptionQuery>;

    /// maps identity name to identity index (simply a set)
    #[pallet::storage]
    #[pallet::getter(fn identity_by_did)]
    pub type IdentitiesNames<T: Config> =
        StorageMap<_, Blake2_128Concat, IdtyName, T::IdtyIndex, OptionQuery>;

    /// counter of the identity index to give to the next identity
    #[pallet::storage]
    pub(super) type NextIdtyIndex<T: Config> = StorageValue<_, T::IdtyIndex, ValueQuery>;

    /// maps block number to the list of identities set to be removed at this bloc
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
                Weight::zero()
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
        /// An identity has been confirmed by its owner
        /// [idty_index, owner_key, name]
        IdtyConfirmed {
            idty_index: T::IdtyIndex,
            owner_key: T::AccountId,
            name: IdtyName,
        },
        /// An identity has been validated
        /// [idty_index]
        IdtyValidated { idty_index: T::IdtyIndex },
        IdtyChangedOwnerKey {
            idty_index: T::IdtyIndex,
            new_owner_key: T::AccountId,
        },
        /// An identity has been removed
        /// [idty_index]
        IdtyRemoved {
            idty_index: T::IdtyIndex,
            reason: IdtyRemovalReason<T::IdtyRemovalOtherReason>,
        },
    }

    // CALLS //

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create an identity for an existing account
        ///
        /// - `owner_key`: the public key corresponding to the identity to be created
        ///
        /// The origin must be allowed to create an identity.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_identity())]
        pub fn create_identity(
            origin: OriginFor<T>,
            owner_key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;

            let creator =
                IdentityIndexOf::<T>::try_get(&who).map_err(|_| Error::<T>::IdtyIndexNotFound)?;
            let creator_idty_val =
                Identities::<T>::try_get(creator).map_err(|_| Error::<T>::IdtyNotFound)?;

            if IdentityIndexOf::<T>::contains_key(&owner_key) {
                return Err(Error::<T>::IdtyAlreadyCreated.into());
            }

            // run checks for identity creation
            T::CheckIdtyCallAllowed::check_create_identity(creator)?;

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
                    data: Default::default(),
                    next_creatable_identity_on: T::BlockNumber::zero(),
                    old_owner_key: None,
                    owner_key: owner_key.clone(),
                    removable_on,
                    status: IdtyStatus::Created,
                },
            );
            IdentitiesRemovableOn::<T>::append(removable_on, (idty_index, IdtyStatus::Created));
            IdentityIndexOf::<T>::insert(owner_key.clone(), idty_index);
            Self::deposit_event(Event::IdtyCreated {
                idty_index,
                owner_key: owner_key.clone(),
            });
            T::OnIdtyChange::on_idty_change(idty_index, &IdtyEvent::Created { creator, owner_key });
            Ok(().into())
        }

        /// Confirm the creation of an identity and give it a name
        ///
        /// - `idty_name`: the name uniquely associated to this identity. Must match the validation rules defined by the runtime.
        ///
        /// The identity must have been created using `create_identity` before it can be confirmed.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::confirm_identity())]
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
            T::CheckIdtyCallAllowed::check_confirm_identity(idty_index)?;

            // Apply phase //
            idty_value.status = IdtyStatus::ConfirmedByOwner;

            <Identities<T>>::insert(idty_index, idty_value);
            <IdentitiesNames<T>>::insert(idty_name.clone(), idty_index);
            Self::deposit_event(Event::IdtyConfirmed {
                idty_index,
                owner_key: who,
                name: idty_name,
            });
            T::OnIdtyChange::on_idty_change(idty_index, &IdtyEvent::Confirmed);
            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::validate_identity())]
        /// validate the owned identity (must meet the main wot requirements)
        // automatically claim membership if not done
        pub fn validate_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let _ = ensure_signed(origin)?;

            let mut idty_value =
                Identities::<T>::try_get(idty_index).map_err(|_| Error::<T>::IdtyNotFound)?;

            match idty_value.status {
                IdtyStatus::Created => return Err(Error::<T>::IdtyNotConfirmedByOwner.into()),
                IdtyStatus::ConfirmedByOwner => {
                    T::CheckIdtyCallAllowed::check_validate_identity(idty_index)?;
                }
                IdtyStatus::Validated => return Err(Error::<T>::IdtyAlreadyValidated.into()),
            }

            // Apply phase //
            idty_value.removable_on = T::BlockNumber::zero();
            idty_value.status = IdtyStatus::Validated;

            <Identities<T>>::insert(idty_index, idty_value);
            Self::deposit_event(Event::IdtyValidated { idty_index });
            T::OnIdtyChange::on_idty_change(idty_index, &IdtyEvent::Validated);

            Ok(().into())
        }

        /// Change identity owner key.
        ///
        /// - `new_key`: the new owner key.
        /// - `new_key_sig`: the signature of the encoded form of `IdtyIndexAccountIdPayload`.
        ///                  Must be signed by `new_key`.
        ///
        /// The origin should be the old identity owner key.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::change_owner_key())]
        pub fn change_owner_key(
            origin: OriginFor<T>,
            new_key: T::AccountId,
            new_key_sig: T::Signature,
        ) -> DispatchResultWithPostInfo {
            // verification phase
            let who = ensure_signed(origin)?;

            let idty_index =
                IdentityIndexOf::<T>::get(&who).ok_or(Error::<T>::IdtyIndexNotFound)?;
            let mut idty_value =
                Identities::<T>::get(idty_index).ok_or(Error::<T>::IdtyNotFound)?;

            ensure!(
                IdentityIndexOf::<T>::get(&new_key).is_none(),
                Error::<T>::OwnerKeyAlreadyUsed
            );

            T::CheckIdtyCallAllowed::check_change_identity_address(idty_index)?;

            let block_number = frame_system::Pallet::<T>::block_number();
            let maybe_old_old_owner_key =
                if let Some((old_owner_key, last_change)) = idty_value.old_owner_key {
                    ensure!(
                        block_number >= last_change + T::ChangeOwnerKeyPeriod::get(),
                        Error::<T>::OwnerKeyAlreadyRecentlyChanged
                    );
                    ensure!(
                        old_owner_key != new_key,
                        Error::<T>::ProhibitedToRevertToAnOldKey
                    );
                    Some(old_owner_key)
                } else {
                    None
                };

            let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
            let new_key_payload = IdtyIndexAccountIdPayload {
                genesis_hash: &genesis_hash,
                idty_index,
                old_owner_key: &idty_value.owner_key,
            };

            ensure!(
                (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload)
                    .using_encoded(|bytes| new_key_sig.verify(bytes, &new_key)),
                Error::<T>::InvalidSignature
            );

            // Apply phase
            if let Some(old_old_owner_key) = maybe_old_old_owner_key {
                frame_system::Pallet::<T>::dec_sufficients(&old_old_owner_key);
            }
            IdentityIndexOf::<T>::remove(&idty_value.owner_key);

            idty_value.old_owner_key = Some((idty_value.owner_key.clone(), block_number));
            idty_value.owner_key = new_key.clone();
            frame_system::Pallet::<T>::inc_sufficients(&idty_value.owner_key);
            IdentityIndexOf::<T>::insert(&idty_value.owner_key, idty_index);
            Identities::<T>::insert(idty_index, idty_value);
            Self::deposit_event(Event::IdtyChangedOwnerKey {
                idty_index,
                new_owner_key: new_key.clone(),
            });
            T::OnIdtyChange::on_idty_change(
                idty_index,
                &IdtyEvent::ChangedOwnerKey {
                    new_owner_key: new_key,
                },
            );

            Ok(().into())
        }

        /// Revoke an identity using a revocation signature
        ///
        /// - `idty_index`: the index of the identity to be revoked.
        /// - `revocation_key`: the key used to sign the revocation payload.
        /// - `revocation_sig`: the signature of the encoded form of `RevocationPayload`.
        ///                     Must be signed by `revocation_key`.
        ///
        /// Any signed origin can execute this call.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::revoke_identity())]
        pub fn revoke_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            revocation_key: T::AccountId,
            revocation_sig: T::Signature,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;

            let idty_value = Identities::<T>::get(idty_index).ok_or(Error::<T>::IdtyNotFound)?;

            ensure!(
                if let Some((ref old_owner_key, last_change)) = idty_value.old_owner_key {
                    // old owner key can also revoke the identity until the period expired
                    revocation_key == idty_value.owner_key
                        || (&revocation_key == old_owner_key
                            && frame_system::Pallet::<T>::block_number()
                                < last_change + T::ChangeOwnerKeyPeriod::get())
                } else {
                    revocation_key == idty_value.owner_key
                },
                Error::<T>::InvalidRevocationKey
            );

            // make sure that no wot prevents identity removal
            T::CheckIdtyCallAllowed::check_remove_identity(idty_index)?;

            // then check payload signature
            let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
            let revocation_payload = RevocationPayload {
                genesis_hash,
                idty_index,
            };

            ensure!(
                (REVOCATION_PAYLOAD_PREFIX, revocation_payload)
                    .using_encoded(|bytes| revocation_sig.verify(bytes, &revocation_key)),
                Error::<T>::InvalidSignature
            );

            // finally if all checks pass, remove identity
            Self::do_remove_identity(idty_index, IdtyRemovalReason::Revoked);
            Ok(().into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::remove_identity())]
        /// remove an identity from storage
        pub fn remove_identity(
            origin: OriginFor<T>,
            idty_index: T::IdtyIndex,
            idty_name: Option<IdtyName>,
            reason: IdtyRemovalReason<T::IdtyRemovalOtherReason>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Self::do_remove_identity(idty_index, reason);
            if let Some(idty_name) = idty_name {
                <IdentitiesNames<T>>::remove(idty_name);
            }

            Ok(().into())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::prune_item_identities_names(names.len() as u32))]
        /// remove identity names from storage
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

        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::fix_sufficients())]
        /// change sufficient ref count for given key
        pub fn fix_sufficients(
            origin: OriginFor<T>,
            owner_key: T::AccountId,
            inc: bool,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            if inc {
                frame_system::Pallet::<T>::inc_sufficients(&owner_key);
            } else {
                frame_system::Pallet::<T>::dec_sufficients(&owner_key);
            }

            Ok(().into())
        }

        /// Link an account to an identity
        // both must sign (target account and identity)
        // can be used for quota system
        // re-uses new owner key payload for simplicity
        // with other custom prefix
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::link_account())]
        pub fn link_account(
            origin: OriginFor<T>,      // origin must have an identity index
            account_id: T::AccountId,  // id of account to link (must sign the payload)
            payload_sig: T::Signature, // signature with linked identity
        ) -> DispatchResultWithPostInfo {
            // verif
            let who = ensure_signed(origin)?;
            let idty_index =
                IdentityIndexOf::<T>::get(&who).ok_or(Error::<T>::IdtyIndexNotFound)?;
            let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
            let payload = IdtyIndexAccountIdPayload {
                genesis_hash: &genesis_hash,
                idty_index,
                old_owner_key: &account_id,
            };
            ensure!(
                (LINK_IDTY_PAYLOAD_PREFIX, payload)
                    .using_encoded(|bytes| payload_sig.verify(bytes, &account_id)),
                Error::<T>::InvalidSignature
            );
            // apply
            Self::do_link_account(account_id, idty_index);

            Ok(().into())
        }
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
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
        /// Identity name already exists
        IdtyNameAlreadyExist,
        /// Invalid identity name
        IdtyNameInvalid,
        /// Identity not confirmed by its owner
        IdtyNotConfirmedByOwner,
        /// Identity not found
        IdtyNotFound,
        /// Identity not member
        IdtyNotMember,
        /// Identity not validated
        IdtyNotValidated,
        /// Identity not yet renewable
        IdtyNotYetRenewable,
        /// payload signature is invalid
        InvalidSignature,
        /// Revocation key is invalid
        InvalidRevocationKey,
        /// Identity creation period is not respected
        NotRespectIdtyCreationPeriod,
        /// Not the same identity name
        NotSameIdtyName,
        /// Owner key already recently changed
        OwnerKeyAlreadyRecentlyChanged,
        /// Owner key already used
        OwnerKeyAlreadyUsed,
        /// Prohibited to revert to an old key
        ProhibitedToRevertToAnOldKey,
        /// Right already added
        RightAlreadyAdded,
        /// Right does not exist
        RightNotExist,
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        pub fn identities_count() -> u32 {
            Identities::<T>::count()
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// try to validate identity
        // (used when membership is claimed first)
        pub fn try_validate_identity(idty_index: T::IdtyIndex) {
            if let Some(mut idty_value) = Identities::<T>::get(idty_index) {
                // only does something if identity is not yet validated
                if idty_value.status != IdtyStatus::Validated {
                    idty_value.removable_on = T::BlockNumber::zero();
                    idty_value.status = IdtyStatus::Validated;

                    <Identities<T>>::insert(idty_index, idty_value);
                    Self::deposit_event(Event::IdtyValidated { idty_index });
                }
                // already validated, no need to re-validate
            }
        }

        /// perform identity removal
        pub(super) fn do_remove_identity(
            idty_index: T::IdtyIndex,
            reason: IdtyRemovalReason<T::IdtyRemovalOtherReason>,
        ) -> Weight {
            if let Some(idty_val) = Identities::<T>::get(idty_index) {
                let _ = T::RemoveIdentityConsumers::remove_idty_consumers(idty_index);
                IdentityIndexOf::<T>::remove(&idty_val.owner_key);
                // Identity should be removed after the consumers of the identity
                Identities::<T>::remove(idty_index);
                frame_system::Pallet::<T>::dec_sufficients(&idty_val.owner_key);
                if let Some((old_owner_key, _last_change)) = idty_val.old_owner_key {
                    frame_system::Pallet::<T>::dec_sufficients(&old_owner_key);
                }
                Self::deposit_event(Event::IdtyRemoved { idty_index, reason });
                T::OnIdtyChange::on_idty_change(
                    idty_index,
                    &IdtyEvent::Removed {
                        status: idty_val.status,
                    },
                );
            }
            Weight::zero()
        }
        /// incremental counter for identity index
        fn get_next_idty_index() -> T::IdtyIndex {
            if let Ok(next_index) = <NextIdtyIndex<T>>::try_get() {
                <NextIdtyIndex<T>>::put(next_index.saturating_add(T::IdtyIndex::one()));
                next_index
            } else {
                <NextIdtyIndex<T>>::put(T::IdtyIndex::one() + T::IdtyIndex::one());
                T::IdtyIndex::one()
            }
        }
        /// remove identities planned for removal at the given block if their status did not change
        fn prune_identities(block_number: T::BlockNumber) -> Weight {
            let mut total_weight = Weight::zero();

            for (idty_index, idty_status) in IdentitiesRemovableOn::<T>::take(block_number) {
                if let Ok(idty_val) = <Identities<T>>::try_get(idty_index) {
                    if idty_val.removable_on == block_number && idty_val.status == idty_status {
                        total_weight +=
                            Self::do_remove_identity(idty_index, IdtyRemovalReason::Expired)
                    }
                }
            }

            total_weight
        }

        /// link account
        fn do_link_account(account_id: T::AccountId, idty_index: T::IdtyIndex) {
            // call account linker
            T::AccountLinker::link_identity(account_id, idty_index);
        }
    }
}

// implement getting owner key of identity index

impl<T: Config> sp_runtime::traits::Convert<T::IdtyIndex, Option<T::AccountId>> for Pallet<T> {
    fn convert(idty_index: T::IdtyIndex) -> Option<T::AccountId> {
        Identities::<T>::get(idty_index).map(|idty_val| idty_val.owner_key)
    }
}

// implement StoredMap trait for this pallet

impl<T> frame_support::traits::StoredMap<T::AccountId, T::IdtyData> for Pallet<T>
where
    T: Config,
{
    /// get identity data for an account id
    fn get(key: &T::AccountId) -> T::IdtyData {
        if let Some(idty_index) = Self::identity_index_of(key) {
            if let Some(idty_val) = Identities::<T>::get(idty_index) {
                idty_val.data
            } else {
                Default::default()
            }
        } else {
            Default::default()
        }
    }
    /// mutate an account given a function of its data
    fn try_mutate_exists<R, E: From<sp_runtime::DispatchError>>(
        key: &T::AccountId,
        f: impl FnOnce(&mut Option<T::IdtyData>) -> Result<R, E>,
    ) -> Result<R, E> {
        let maybe_idty_index = Self::identity_index_of(key);
        let mut maybe_idty_data = if let Some(idty_index) = maybe_idty_index {
            if let Some(idty_val) = Identities::<T>::get(idty_index) {
                Some(idty_val.data)
            } else {
                None
            }
        } else {
            None
        };
        let result = f(&mut maybe_idty_data)?;
        if let Some(idty_index) = maybe_idty_index {
            Identities::<T>::mutate_exists(idty_index, |idty_val_opt| {
                if let Some(ref mut idty_val) = idty_val_opt {
                    idty_val.data = maybe_idty_data.unwrap_or_default();
                } else if maybe_idty_data.is_some() {
                    return Err(sp_runtime::DispatchError::Other(
                        "Tring to set IdtyData for a non-existing identity!",
                    ));
                }
                Ok(())
            })?;
        } else if maybe_idty_data.is_some() {
            return Err(sp_runtime::DispatchError::Other(
                "Tring to set IdtyData for a non-existing identity!",
            )
            .into());
        }
        Ok(result)
    }
}
