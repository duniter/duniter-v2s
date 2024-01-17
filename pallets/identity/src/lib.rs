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
        /// Period during which the owner can confirm the new identity.
        // something like 2 days but this should be done quickly as the first certifier is helping
        #[pallet::constant]
        type ConfirmPeriod: Get<Self::BlockNumber>;
        /// Period before which the identity has to be validated (become member).
        // this is the 2 month period in v1
        #[pallet::constant]
        type ValidationPeriod: Get<Self::BlockNumber>;
        /// Period before which an identity who lost membership is automatically revoked.
        // this is the 1 year period in v1
        #[pallet::constant]
        type AutorevocationPeriod: Get<Self::BlockNumber>;
        /// Period after which a revoked identity is removed and the keys are freed.
        #[pallet::constant]
        type DeletionPeriod: Get<Self::BlockNumber>;
        /// Minimum duration between two owner key changes.
        // to avoid stealing the identity without means to revoke
        #[pallet::constant]
        type ChangeOwnerKeyPeriod: Get<Self::BlockNumber>;
        /// Minimum duration between the creation of 2 identities by the same creator.
        // it should be greater or equal than the certification period in certification pallet
        #[pallet::constant]
        type IdtyCreationPeriod: Get<Self::BlockNumber>;
        /// Management of the authorizations of the different calls.
        /// The default implementation allows everything.
        type CheckIdtyCallAllowed: CheckIdtyCallAllowed<Self>;
        /// Custom data to store in each identity.
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
        /// On identity confirmed by its owner
        type OnIdtyChange: OnIdtyChange<Self>;
        /// Signing key of a payload
        type Signer: IdentifyAccount<AccountId = Self::AccountId>;
        /// Signature of a payload
        type Signature: Parameter + Verify<Signer = Self::Signer>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
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
                if idty.value.next_scheduled > T::BlockNumber::zero() {
                    <IdentityChangeSchedule<T>>::append(idty.value.next_scheduled, idty_index)
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
    #[pallet::getter(fn next_scheduled)]
    pub type IdentityChangeSchedule<T: Config> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::IdtyIndex>, ValueQuery>;

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if n > T::BlockNumber::zero() {
                Self::prune_identities(n).saturating_add(T::WeightInfo::on_initialize())
            } else {
                T::WeightInfo::on_initialize()
            }
        }
    }

    // EVENTS //

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new identity has been created.
        IdtyCreated {
            idty_index: T::IdtyIndex,
            owner_key: T::AccountId,
        },
        /// An identity has been confirmed by its owner.
        IdtyConfirmed {
            idty_index: T::IdtyIndex,
            owner_key: T::AccountId,
            name: IdtyName,
        },
        /// An identity has been validated.
        IdtyValidated { idty_index: T::IdtyIndex },
        IdtyChangedOwnerKey {
            idty_index: T::IdtyIndex,
            new_owner_key: T::AccountId,
        },
        /// An identity has been revoked.
        IdtyRevoked {
            idty_index: T::IdtyIndex,
            reason: RevocationReason,
        },
        /// An identity has been removed.
        IdtyRemoved {
            idty_index: T::IdtyIndex,
            reason: RemovalReason,
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

            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let creator_index = Self::check_create_identity(&who, &owner_key, block_number)?;

            // Apply phase //
            frame_system::Pallet::<T>::inc_sufficients(&owner_key);
            <Identities<T>>::mutate_exists(creator_index, |idty_val_opt| {
                if let Some(ref mut idty_val) = idty_val_opt {
                    idty_val.next_creatable_identity_on =
                        block_number + T::IdtyCreationPeriod::get();
                }
            });

            let idty_index = Self::get_next_idty_index();
            <Identities<T>>::insert(
                idty_index,
                IdtyValue {
                    data: Default::default(),
                    next_creatable_identity_on: T::BlockNumber::zero(),
                    old_owner_key: None,
                    owner_key: owner_key.clone(),
                    next_scheduled: Self::schedule_identity_change(
                        idty_index,
                        T::ConfirmPeriod::get(),
                    ),
                    status: IdtyStatus::Unconfirmed,
                },
            );

            IdentityIndexOf::<T>::insert(owner_key.clone(), idty_index);
            Self::deposit_event(Event::IdtyCreated {
                idty_index,
                owner_key: owner_key.clone(),
            });
            T::AccountLinker::link_identity(&owner_key, idty_index)?;
            T::OnIdtyChange::on_idty_change(
                idty_index,
                &IdtyEvent::Created {
                    creator: creator_index,
                    owner_key,
                },
            );
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

            let idty_value =
                Identities::<T>::try_get(idty_index).map_err(|_| Error::<T>::IdtyNotFound)?;

            if idty_value.status != IdtyStatus::Unconfirmed {
                return Err(Error::<T>::IdtyAlreadyConfirmed.into());
            }
            if !T::IdtyNameValidator::validate(&idty_name) {
                return Err(Error::<T>::IdtyNameInvalid.into());
            }
            if <IdentitiesNames<T>>::contains_key(&idty_name) {
                return Err(Error::<T>::IdtyNameAlreadyExist.into());
            }

            // Apply phase //
            Self::update_identity_status(
                idty_index,
                idty_value,
                IdtyStatus::Unvalidated,
                T::ValidationPeriod::get(),
            );

            <IdentitiesNames<T>>::insert(idty_name.clone(), idty_index);
            Self::deposit_event(Event::IdtyConfirmed {
                idty_index,
                owner_key: who,
                name: idty_name,
            });
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
            T::AccountLinker::link_identity(&new_key, idty_index)?;
            Self::deposit_event(Event::IdtyChangedOwnerKey {
                idty_index,
                new_owner_key: new_key,
            });

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

            match idty_value.status {
                IdtyStatus::Unconfirmed => return Err(Error::<T>::CanNotRevokeUnconfirmed.into()),
                IdtyStatus::Unvalidated => return Err(Error::<T>::CanNotRevokeUnvalidated.into()),
                IdtyStatus::Member => (),
                IdtyStatus::NotMember => (),
                IdtyStatus::Revoked => return Err(Error::<T>::AlreadyRevoked.into()),
            };

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
            Self::do_revoke_identity(idty_index, RevocationReason::User);
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
            T::AccountLinker::link_identity(&account_id, idty_index)?;

            Ok(().into())
        }
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Identity already confirmed.
        IdtyAlreadyConfirmed,
        /// Identity already created.
        IdtyAlreadyCreated,
        /// Identity already validated.
        IdtyAlreadyValidated,
        /// Identity index not found.
        IdtyIndexNotFound,
        /// Identity name already exists.
        IdtyNameAlreadyExist,
        /// Invalid identity name.
        IdtyNameInvalid,
        /// Identity not confirmed by its owner.
        IdtyNotConfirmed,
        /// Identity not found.
        IdtyNotFound,
        /// Invalid payload signature.
        InvalidSignature,
        /// Invalid revocation key.
        InvalidRevocationKey,
        /// Issuer is not member and can not perform this action.
        IssuerNotMember,
        /// Identity creation period is not respected.
        NotRespectIdtyCreationPeriod,
        /// Owner key already changed recently.
        OwnerKeyAlreadyRecentlyChanged,
        /// Owner key already used.
        OwnerKeyAlreadyUsed,
        /// Reverting to an old key is prohibited.
        ProhibitedToRevertToAnOldKey,
        /// Already revoked.
        AlreadyRevoked,
        /// Can not revoke identity that never was member.
        CanNotRevokeUnconfirmed,
        /// Can not revoke identity that never was member.
        CanNotRevokeUnvalidated,
        /// Cannot link to an inexisting account.
        AccountNotExist,
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// get identity count
        pub fn identities_count() -> u32 {
            Identities::<T>::count()
        }

        /// membership added
        // when an identity becomes member, update its status
        // unschedule identity action, this falls back to membership scheduling
        // no identity schedule while membership is active
        pub fn membership_added(idty_index: T::IdtyIndex) {
            if let Some(mut idty_value) = Identities::<T>::get(idty_index) {
                Self::unschedule_identity_change(idty_index, idty_value.next_scheduled);
                idty_value.next_scheduled = T::BlockNumber::zero();
                idty_value.status = IdtyStatus::Member;
                <Identities<T>>::insert(idty_index, idty_value);
                Self::deposit_event(Event::IdtyValidated { idty_index });
            }
            // else should not happen
        }

        /// membership removed
        // only does something if identity is actually member
        // a membership can be removed when the identity is revoked
        // in this case, this does nothing
        pub fn membership_removed(idty_index: T::IdtyIndex) {
            if let Some(idty_value) = Identities::<T>::get(idty_index) {
                if idty_value.status == IdtyStatus::Member {
                    Self::update_identity_status(
                        idty_index,
                        idty_value,
                        IdtyStatus::NotMember,
                        T::AutorevocationPeriod::get(),
                    );
                }
            }
            // else should not happen
        }

        /// perform identity removal
        // (kind of garbage collector)
        // this should not be called while the identity is still member
        // otherwise there will still be a membership in storage, but no more identity
        pub fn do_remove_identity(idty_index: T::IdtyIndex, reason: RemovalReason) -> Weight {
            if let Some(idty_value) = Identities::<T>::get(idty_index) {
                // this line allows the owner key to be used after that
                IdentityIndexOf::<T>::remove(&idty_value.owner_key);
                // Identity should be removed after the consumers of the identity
                Identities::<T>::remove(idty_index);
                frame_system::Pallet::<T>::dec_sufficients(&idty_value.owner_key);
                if let Some((old_owner_key, _last_change)) = idty_value.old_owner_key {
                    frame_system::Pallet::<T>::dec_sufficients(&old_owner_key);
                }
                Self::deposit_event(Event::IdtyRemoved { idty_index, reason });
                T::OnIdtyChange::on_idty_change(
                    idty_index,
                    &IdtyEvent::Removed {
                        status: idty_value.status,
                    },
                );
                return T::WeightInfo::do_remove_identity();
            }
            T::WeightInfo::do_remove_identity_noop()
        }

        /// revoke identity
        pub fn do_revoke_identity(idty_index: T::IdtyIndex, reason: RevocationReason) -> Weight {
            if let Some(idty_value) = Identities::<T>::get(idty_index) {
                Self::update_identity_status(
                    idty_index,
                    idty_value,
                    IdtyStatus::Revoked,
                    T::DeletionPeriod::get(),
                );

                Self::deposit_event(Event::IdtyRevoked { idty_index, reason });
                T::OnIdtyChange::on_idty_change(
                    idty_index,
                    &IdtyEvent::Removed {
                        status: IdtyStatus::Revoked,
                    },
                );
                return T::WeightInfo::do_revoke_identity();
            }
            T::WeightInfo::do_revoke_identity_noop()
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

        /// remove identities planned for removal at the given block
        pub fn prune_identities(block_number: T::BlockNumber) -> Weight {
            let mut total_weight = Weight::zero();

            for idty_index in IdentityChangeSchedule::<T>::take(block_number) {
                if let Ok(idty_val) = <Identities<T>>::try_get(idty_index) {
                    if idty_val.next_scheduled == block_number {
                        match idty_val.status {
                            IdtyStatus::Unconfirmed => {
                                total_weight +=
                                    Self::do_remove_identity(idty_index, RemovalReason::Unconfirmed)
                            }
                            IdtyStatus::Unvalidated => {
                                total_weight +=
                                    Self::do_remove_identity(idty_index, RemovalReason::Unvalidated)
                            }
                            IdtyStatus::Revoked => {
                                total_weight +=
                                    Self::do_remove_identity(idty_index, RemovalReason::Revoked)
                            }
                            IdtyStatus::NotMember => {
                                total_weight +=
                                    Self::do_revoke_identity(idty_index, RevocationReason::Expired)
                            }
                            IdtyStatus::Member => { // do not touch identities of member accounts
                                 // this should not happen
                            }
                        }
                    } else {
                        total_weight += T::WeightInfo::prune_identities_err()
                            .saturating_sub(T::WeightInfo::prune_identities_none())
                    }
                } else {
                    total_weight += T::WeightInfo::prune_identities_none()
                        .saturating_sub(T::WeightInfo::prune_identities_noop())
                }
            }

            total_weight.saturating_add(T::WeightInfo::prune_identities_noop())
        }

        /// change identity status and reschedule next action
        fn update_identity_status(
            idty_index: T::IdtyIndex,
            mut idty_value: IdtyValue<T::BlockNumber, T::AccountId, T::IdtyData>,
            new_status: IdtyStatus,
            period: T::BlockNumber,
        ) {
            Self::unschedule_identity_change(idty_index, idty_value.next_scheduled);
            idty_value.next_scheduled = Self::schedule_identity_change(idty_index, period);
            idty_value.status = new_status;
            <Identities<T>>::insert(idty_index, idty_value);
        }

        /// unschedule identity change
        fn unschedule_identity_change(idty_id: T::IdtyIndex, block_number: T::BlockNumber) {
            let mut scheduled = IdentityChangeSchedule::<T>::get(block_number);
            if let Some(pos) = scheduled.iter().position(|x| *x == idty_id) {
                scheduled.swap_remove(pos);
                IdentityChangeSchedule::<T>::set(block_number, scheduled);
            }
        }
        /// schedule identity change after given period
        fn schedule_identity_change(
            idty_id: T::IdtyIndex,
            period: T::BlockNumber,
        ) -> T::BlockNumber {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let next_scheduled = block_number + period;
            IdentityChangeSchedule::<T>::append(next_scheduled, idty_id);
            next_scheduled
        }

        /// check create identity
        // first internal checks
        // then other pallet checks trough trait
        fn check_create_identity(
            issuer_key: &T::AccountId,
            receiver_key: &T::AccountId,
            block_number: T::BlockNumber,
        ) -> Result<T::IdtyIndex, DispatchError> {
            // first get issuer details
            let creator_index = IdentityIndexOf::<T>::try_get(issuer_key)
                .map_err(|_| Error::<T>::IdtyIndexNotFound)?;
            let creator_idty_val =
                Identities::<T>::try_get(creator_index).map_err(|_| Error::<T>::IdtyNotFound)?;

            // --- some checks can be done internally
            // 1. issuer is member
            ensure!(
                creator_idty_val.status == IdtyStatus::Member,
                Error::<T>::IssuerNotMember
            );

            // 2. issuer respects identity creation period
            ensure!(
                creator_idty_val.next_creatable_identity_on <= block_number,
                Error::<T>::NotRespectIdtyCreationPeriod
            );

            // 3. receiver key is not already used by another identity
            ensure!(
                !IdentityIndexOf::<T>::contains_key(receiver_key),
                Error::<T>::IdtyAlreadyCreated
            );

            // --- other checks depend on other pallets
            // run checks for identity creation
            T::CheckIdtyCallAllowed::check_create_identity(creator_index)?;

            Ok(creator_index)
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
