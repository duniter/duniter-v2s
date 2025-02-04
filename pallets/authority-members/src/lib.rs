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

//! # Duniter Authority Members Pallet
//!
//! In a permissioned network, defining the set of authorities and selecting validators for the next session is crucial.
//! The authority members pallet is responsible for this. Specifically, it:
//!
//! - Manages a `Members` set with custom rules.
//! - Implements the `SessionManager` trait from the FRAME session pallet.
//!
//! ## Entering the Set of Authorities
//!
//! To become part of Duniter authorities, one must follow these steps:
//!
//! 1. Become a member of the main web of trust.
//! 2. Request membership to the smith sub web of trust.
//! 3. Obtain enough certifications to gain smith membership.
//! 4. Claim membership to the set of authorities.
//!
//! After becoming an authority, one can "go online" and "go offline" to enter or leave two sessions later.
//!
//! ## Some Vocabulary
//!
//! *Smiths* are individuals allowed to forge blocks. Specifically, this entails:
//!
//! - **Smith** status is required to become an authority.
//! - **Authority** status is required to become a validator.
//! - **Validator** status is required to add blocks.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

pub mod impls;
pub mod traits;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

pub use pallet::*;
pub use sp_staking::SessionIndex;
pub use traits::*;
pub use types::*;
pub use weights::WeightInfo;

use codec::alloc::borrow::ToOwned;
use frame_support::traits::Get;
use scale_info::prelude::{collections::BTreeMap, vec, vec::Vec};
use sp_runtime::traits::{Convert, IsMember};

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{StorageVersion, UnfilteredDispatchable, ValidatorRegistration},
    };
    use frame_system::pallet_prelude::*;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_session::Config + pallet_session::historical::Config
    {
        /// Specifies the type that determines membership status.
        type IsMember: IsMember<Self::MemberId>;

        /// Handler for when a new session is initiated.
        type OnNewSession: OnNewSession;

        /// Handler for when a member is removed from authorities.
        type OnOutgoingMember: OnOutgoingMember<Self::MemberId>;

        /// Handler for when a new member is added to authorities.
        type OnIncomingMember: OnIncomingMember<Self::MemberId>;

        /// Maximum number of authorities allowed.
        #[pallet::constant]
        type MaxAuthorities: Get<u32>;

        /// Type representing the identifier of a member.
        type MemberId: Copy + Ord + MaybeSerializeDeserialize + Parameter;

        /// Converts an `AccountId` to an optional `MemberId`.
        type MemberIdOf: Convert<Self::AccountId, Option<Self::MemberId>>;

        /// Specifies the origin type required to remove a member.
        type RemoveMemberOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_authorities: BTreeMap<T::MemberId, (T::AccountId, bool)>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_authorities: BTreeMap::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (member_id, (account_id, _is_online)) in &self.initial_authorities {
                Members::<T>::insert(member_id, MemberData::new_genesis(account_id.to_owned()));
            }
            let mut members_ids = self
                .initial_authorities
                .iter()
                .filter_map(
                    |(member_id, (_account_id, is_online))| {
                        if *is_online {
                            Some(*member_id)
                        } else {
                            None
                        }
                    },
                )
                .collect::<Vec<T::MemberId>>();
            members_ids.sort();

            OnlineAuthorities::<T>::put(members_ids.clone());
        }
    }

    // STORAGE //

    /// The incoming authorities.
    #[pallet::storage]
    #[pallet::getter(fn incoming)]
    pub type IncomingAuthorities<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    /// The online authorities.
    #[pallet::storage]
    #[pallet::getter(fn online)]
    pub type OnlineAuthorities<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    /// The outgoing authorities.
    #[pallet::storage]
    #[pallet::getter(fn outgoing)]
    pub type OutgoingAuthorities<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    /// The member data.
    #[pallet::storage]
    #[pallet::getter(fn member)]
    pub type Members<T: Config> =
        StorageMap<_, Twox64Concat, T::MemberId, MemberData<T::AccountId>, OptionQuery>;

    /// The blacklisted authorities.
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    // HOOKS //

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// List of members scheduled to join the set of authorities in the next session.
        IncomingAuthorities { members: Vec<T::MemberId> },
        /// List of members leaving the set of authorities in the next session.
        OutgoingAuthorities { members: Vec<T::MemberId> },
        /// A member will leave the set of authorities in 2 sessions.
        MemberGoOffline { member: T::MemberId },
        /// A member will join the set of authorities in 2 sessions.
        MemberGoOnline { member: T::MemberId },
        /// A member, who no longer has authority rights, will be removed from the authority set in 2 sessions.
        MemberRemoved { member: T::MemberId },
        /// A member has been removed from the blacklist.
        MemberRemovedFromBlacklist { member: T::MemberId },
        /// A member has been blacklisted.
        MemberAddedToBlacklist { member: T::MemberId },
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Member already incoming.
        AlreadyIncoming,
        /// Member already online.
        AlreadyOnline,
        /// Member already outgoing.
        AlreadyOutgoing,
        /// Owner key is invalid as a member.
        MemberIdNotFound,
        /// Member is blacklisted.
        MemberBlacklisted,
        /// Member is not blacklisted.
        MemberNotBlacklisted,
        /// Member not found.
        MemberNotFound,
        /// Neither online nor scheduled.
        NotOnlineNorIncoming,
        /// Not member.
        NotMember,
        /// Session keys not provided.
        SessionKeysNotProvided,
        /// Too many authorities.
        TooManyAuthorities,
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request to leave the set of validators two sessions later.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::go_offline())]
        pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;
            let member_id = Self::verify_ownership_and_membership(&who)?;

            if !Members::<T>::contains_key(member_id) {
                return Err(Error::<T>::MemberNotFound.into());
            }
            if Self::is_outgoing(member_id) {
                return Err(Error::<T>::AlreadyOutgoing.into());
            }
            let is_incoming = Self::is_incoming(member_id);
            if !is_incoming && !Self::is_online(member_id) {
                return Err(Error::<T>::NotOnlineNorIncoming.into());
            }

            // Apply phase //
            if is_incoming {
                Self::remove_in(member_id);
            } else {
                Self::insert_out(member_id);
            }

            Ok(().into())
        }

        /// Request to join the set of validators two sessions later.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::go_online())]
        pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;
            let member_id = Self::verify_ownership_and_membership(&who)?;

            if Self::is_blacklisted(member_id) {
                return Err(Error::<T>::MemberBlacklisted.into());
            }
            if !Members::<T>::contains_key(member_id) {
                return Err(Error::<T>::MemberNotFound.into());
            }
            let validator_id = T::ValidatorIdOf::convert(who)
                .ok_or(pallet_session::Error::<T>::NoAssociatedValidatorId)?;
            if !pallet_session::Pallet::<T>::is_registered(&validator_id) {
                return Err(Error::<T>::SessionKeysNotProvided.into());
            }

            if Self::is_incoming(member_id) {
                return Err(Error::<T>::AlreadyIncoming.into());
            }
            let is_outgoing = Self::is_outgoing(member_id);
            if Self::is_online(member_id) && !is_outgoing {
                return Err(Error::<T>::AlreadyOnline.into());
            }
            if Self::authorities_counter() >= T::MaxAuthorities::get() {
                return Err(Error::<T>::TooManyAuthorities.into());
            }

            // Apply phase //
            if is_outgoing {
                Self::remove_out(member_id);
            } else {
                Self::insert_in(member_id);
            }

            Ok(().into())
        }

        /// Declare new session keys to replace current ones.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_session_keys())]
        pub fn set_session_keys(origin: OriginFor<T>, keys: T::Keys) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            let member_id = Self::verify_ownership_and_membership(&who)?;

            let _post_info = pallet_session::Call::<T>::set_keys {
                keys,
                proof: vec![],
            }
            .dispatch_bypass_filter(origin)?;

            Members::<T>::insert(member_id, MemberData { owner_key: who });

            Ok(().into())
        }

        /// Remove a member from the set of validators.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_member())]
        pub fn remove_member(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            T::RemoveMemberOrigin::ensure_origin(origin)?;

            let member_data = Members::<T>::get(member_id).ok_or(Error::<T>::MemberNotFound)?;
            Self::do_remove_member(member_id, member_data.owner_key);

            Ok(().into())
        }

        /// Remove a member from the blacklist.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_member_from_blacklist())]
        /// remove an identity from the blacklist
        pub fn remove_member_from_blacklist(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            T::RemoveMemberOrigin::ensure_origin(origin)?;
            Blacklist::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                    Self::deposit_event(Event::MemberRemovedFromBlacklist { member: member_id });
                    Ok(().into())
                } else {
                    Err(Error::<T>::MemberNotBlacklisted.into())
                }
            })
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        // Should be transactional (if an error occurs, storage should not be modified at all)
        //  Execute the annotated function in a new storage transaction.
        //  The return type of the annotated function must be `Result`. All changes to storage performed
        //  by the annotated function are discarded if it returns `Err`, or committed if `Ok`.
        #[frame_support::transactional]
        /// Change the owner key of an authority member.
        pub fn change_owner_key(
            member_id: T::MemberId,
            new_owner_key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let old_owner_key = Members::<T>::mutate_exists(member_id, |maybe_member_data| {
                if let Some(ref mut member_data) = maybe_member_data {
                    Ok(core::mem::replace(
                        &mut member_data.owner_key,
                        new_owner_key.clone(),
                    ))
                } else {
                    Err(Error::<T>::MemberNotFound)
                }
            })?;

            let validator_id = T::ValidatorIdOf::convert(old_owner_key.clone())
                .ok_or(pallet_session::Error::<T>::NoAssociatedValidatorId)?;
            let session_keys = pallet_session::NextKeys::<T>::get(validator_id)
                .ok_or(Error::<T>::SessionKeysNotProvided)?;

            // Purge session keys
            let _post_info = pallet_session::Call::<T>::purge_keys {}
                .dispatch_bypass_filter(frame_system::RawOrigin::Signed(old_owner_key).into())?;

            // Set session keys
            let _post_info = pallet_session::Call::<T>::set_keys {
                keys: session_keys,
                proof: vec![],
            }
            .dispatch_bypass_filter(frame_system::RawOrigin::Signed(new_owner_key).into())?;

            Ok(().into())
        }

        /// Get the number of authorities.
        pub fn authorities_counter() -> u32 {
            let count = OnlineAuthorities::<T>::get().len() + IncomingAuthorities::<T>::get().len()
                - OutgoingAuthorities::<T>::get().len();
            count as u32
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Perform authority member removal.
        fn do_remove_member(member_id: T::MemberId, owner_key: T::AccountId) {
            if Self::is_online(member_id) {
                // Trigger the member deletion for next session
                Self::insert_out(member_id);
            }

            // remove all member data
            Self::remove_in(member_id);
            Self::remove_online(member_id);
            Members::<T>::remove(member_id);

            // Purge session keys
            if let Err(e) = pallet_session::Pallet::<T>::purge_keys(
                frame_system::Origin::<T>::Signed(owner_key).into(),
            ) {
                log::error!(
                    target: "runtime::authority_members",
                    "Logic error: fail to purge session keys in do_remove_member(): {:?}",
                    e
                );
            }

            // Emit event
            Self::deposit_event(Event::MemberRemoved { member: member_id });
        }

        /// Perform incoming authorities insertion.
        fn insert_in(member_id: T::MemberId) -> bool {
            let not_already_inserted = IncomingAuthorities::<T>::mutate(|members_ids| {
                if let Err(index) = members_ids.binary_search(&member_id) {
                    members_ids.insert(index, member_id);
                    true
                } else {
                    false
                }
            });
            if not_already_inserted {
                Self::deposit_event(Event::MemberGoOnline { member: member_id });
            }
            not_already_inserted
        }

        /// Perform outgoing authority insertion.
        pub fn insert_out(member_id: T::MemberId) -> bool {
            let not_already_inserted = OutgoingAuthorities::<T>::mutate(|members_ids| {
                if let Err(index) = members_ids.binary_search(&member_id) {
                    members_ids.insert(index, member_id);
                    true
                } else {
                    false
                }
            });
            if not_already_inserted {
                Self::deposit_event(Event::MemberGoOffline { member: member_id });
            }
            not_already_inserted
        }

        /// Check if member is incoming.
        fn is_incoming(member_id: T::MemberId) -> bool {
            IncomingAuthorities::<T>::get()
                .binary_search(&member_id)
                .is_ok()
        }

        /// C&heck if member is online.
        fn is_online(member_id: T::MemberId) -> bool {
            OnlineAuthorities::<T>::get()
                .binary_search(&member_id)
                .is_ok()
        }

        /// Check if member is outgoing.
        fn is_outgoing(member_id: T::MemberId) -> bool {
            OutgoingAuthorities::<T>::get()
                .binary_search(&member_id)
                .is_ok()
        }

        /// Check if member is blacklisted.
        fn is_blacklisted(member_id: T::MemberId) -> bool {
            Blacklist::<T>::get().contains(&member_id)
        }

        /// Perform removal from incoming authorities.
        fn remove_in(member_id: T::MemberId) {
            IncomingAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            })
        }

        /// Perform removal from online authorities.
        fn remove_online(member_id: T::MemberId) {
            OnlineAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            });
        }

        /// Perform removal from outgoing authorities.
        fn remove_out(member_id: T::MemberId) {
            OutgoingAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            });
        }

        /// Check that account is member.
        fn verify_ownership_and_membership(
            who: &T::AccountId,
        ) -> Result<T::MemberId, DispatchError> {
            let member_id =
                T::MemberIdOf::convert(who.clone()).ok_or(Error::<T>::MemberIdNotFound)?;

            if !T::IsMember::is_member(&member_id) {
                return Err(Error::<T>::NotMember.into());
            }

            Ok(member_id)
        }
    }
}

impl<T: Config> pallet_session::SessionManager<T::ValidatorId> for Pallet<T> {
    /// Plan a new session, and optionally provide the new validator set.
    ///
    /// Even if the validator-set is the same as before, if any underlying economic conditions have
    /// changed (i.e. stake-weights), the new validator set must be returned. This is necessary for
    /// consensus engines making use of the session pallet to issue a validator-set change so
    /// misbehavior can be provably associated with the new economic conditions as opposed to the
    /// old. The returned validator set, if any, will not be applied until `new_index`. `new_index`
    /// is strictly greater than from previous call.
    ///
    /// The first session start at index 0.
    ///
    /// `new_session(session)` is guaranteed to be called before `end_session(session-1)`. In other
    /// words, a new session must always be planned before an ongoing one can be finished.
    fn new_session(_session_index: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        let members_ids_to_add = IncomingAuthorities::<T>::take();
        let members_ids_to_del = OutgoingAuthorities::<T>::take();

        if members_ids_to_add.is_empty() && members_ids_to_del.is_empty() {
            // when no change to the set of autorities, return None
            return None;
        }

        // -- handle incoming members
        // callback when smith is incoming
        for member_id in members_ids_to_add.iter() {
            T::OnIncomingMember::on_incoming_member(*member_id);
        }
        // a single event with all authorities if some
        if !members_ids_to_add.is_empty() {
            Self::deposit_event(Event::IncomingAuthorities {
                members: members_ids_to_add.clone(),
            });
        }

        // -- handle outgoing members
        // callback when smith is outgoing
        for member_id in members_ids_to_del.iter() {
            T::OnOutgoingMember::on_outgoing_member(*member_id);
        }
        // a single event with all authorities if some
        if !members_ids_to_del.is_empty() {
            Self::deposit_event(Event::OutgoingAuthorities {
                members: members_ids_to_del.clone(),
            });
        }

        // updates the list of OnlineAuthorities and returns the list of their key
        Some(
            OnlineAuthorities::<T>::mutate(|members_ids| {
                for member_id in members_ids_to_del {
                    if let Ok(index) = members_ids.binary_search(&member_id) {
                        members_ids.remove(index);
                    }
                }
                for member_id in members_ids_to_add {
                    if let Err(index) = members_ids.binary_search(&member_id) {
                        members_ids.insert(index, member_id);
                    }
                }
                members_ids.clone()
            })
            .into_iter()
            .filter_map(|member_id| {
                if let Some(member_data) = Members::<T>::get(member_id) {
                    T::ValidatorIdOf::convert(member_data.owner_key)
                } else {
                    None
                }
            })
            .collect(),
        )
    }

    /// Same as `new_session`, but it this should only be called at genesis.
    fn new_session_genesis(_new_index: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        Some(
            OnlineAuthorities::<T>::get()
                .into_iter()
                .filter_map(|member_id| {
                    if let Some(member_data) = Members::<T>::get(member_id) {
                        T::ValidatorIdOf::convert(member_data.owner_key)
                    } else {
                        None
                    }
                })
                .collect(),
        )
    }

    /// End the session.
    ///
    /// Because the session pallet can queue validator set the ending session can be lower than the
    /// last new session index.
    fn end_session(_end_index: SessionIndex) {}

    /// Start an already planned session.
    ///
    /// The session start to be used for validation.
    fn start_session(start_index: SessionIndex) {
        T::OnNewSession::on_new_session(start_index);
    }
}

// see substrate FullIdentification
fn add_full_identification<T: Config>(
    validator_id: T::ValidatorId,
) -> Option<(T::ValidatorId, T::FullIdentification)> {
    use sp_runtime::traits::Convert as _;
    T::FullIdentificationOf::convert(validator_id.clone())
        .map(|full_ident| (validator_id, full_ident))
}

// implement SessionManager with FullIdentification
impl<T: Config> pallet_session::historical::SessionManager<T::ValidatorId, T::FullIdentification>
    for Pallet<T>
{
    fn new_session(
        new_index: SessionIndex,
    ) -> Option<Vec<(T::ValidatorId, T::FullIdentification)>> {
        <Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators_ids| {
            validators_ids
                .into_iter()
                .filter_map(add_full_identification::<T>)
                .collect()
        })
    }

    fn new_session_genesis(
        new_index: SessionIndex,
    ) -> Option<Vec<(T::ValidatorId, T::FullIdentification)>> {
        <Self as pallet_session::SessionManager<_>>::new_session_genesis(new_index).map(
            |validators_ids| {
                validators_ids
                    .into_iter()
                    .filter_map(add_full_identification::<T>)
                    .collect()
            },
        )
    }

    fn start_session(start_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::start_session(start_index)
    }

    fn end_session(end_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::end_session(end_index)
    }
}
