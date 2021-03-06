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

use self::traits::*;
use frame_support::traits::Get;
use sp_runtime::traits::Convert;
use sp_staking::SessionIndex;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::ValidatorRegistration;
    use frame_support::traits::{StorageVersion, UnfilteredDispatchable};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Convert, IsMember};
    use sp_std::collections::btree_map::BTreeMap;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_session::Config + pallet_session::historical::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type KeysWrapper: Parameter + Into<Self::Keys>;
        type IsMember: IsMember<Self::MemberId>;
        type OnNewSession: OnNewSession;
        type OnRemovedMember: OnRemovedMember<Self::MemberId>;
        /// Max number of authorities allowed
        #[pallet::constant]
        type MaxAuthorities: Get<u32>;
        #[pallet::constant]
        type MaxKeysLife: Get<SessionIndex>;
        #[pallet::constant]
        type MaxOfflineSessions: Get<SessionIndex>;
        type MemberId: Copy + Ord + MaybeSerializeDeserialize + Parameter;
        type MemberIdOf: Convert<Self::AccountId, Option<Self::MemberId>>;
        type RemoveMemberOrigin: EnsureOrigin<Self::Origin>;
    }

    // GENESIS STUFF??//

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_authorities: BTreeMap<T::MemberId, (T::AccountId, bool)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_authorities: BTreeMap::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (member_id, (account_id, _is_online)) in &self.initial_authorities {
                Members::<T>::insert(
                    member_id,
                    MemberData::new_genesis(T::MaxKeysLife::get(), account_id.to_owned()),
                );
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

            AuthoritiesCounter::<T>::put(members_ids.len() as u32);
            OnlineAuthorities::<T>::put(members_ids.clone());
            MustRotateKeysBefore::<T>::insert(T::MaxKeysLife::get(), members_ids);
        }
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn account_id_of)]
    pub type AccountIdOf<T: Config> =
        StorageMap<_, Twox64Concat, T::MemberId, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn authorities_counter)]
    pub type AuthoritiesCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn incoming)]
    pub type IncomingAuthorities<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn online)]
    pub type OnlineAuthorities<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn outgoing)]
    pub type OutgoingAuthorities<T: Config> = StorageValue<_, Vec<T::MemberId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn member)]
    pub type Members<T: Config> =
        StorageMap<_, Twox64Concat, T::MemberId, MemberData<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn members_expire_on)]
    pub type MembersExpireOn<T: Config> =
        StorageMap<_, Twox64Concat, SessionIndex, Vec<T::MemberId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn must_rotate_keys_before)]
    pub type MustRotateKeysBefore<T: Config> =
        StorageMap<_, Twox64Concat, SessionIndex, Vec<T::MemberId>, ValueQuery>;

    // HOOKS //

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// List of members who will enter the set of authorities at the next session.
        /// [Vec<member_id>]
        IncomingAuthorities(Vec<T::MemberId>),
        /// List of members who will leave the set of authorities at the next session.
        /// [Vec<member_id>]
        OutgoingAuthorities(Vec<T::MemberId>),
        /// A member will leave the set of authorities in 2 sessions.
        /// [member_id]
        MemberGoOffline(T::MemberId),
        /// A member will enter the set of authorities in 2 sessions.
        /// [member_id]
        MemberGoOnline(T::MemberId),
        /// A member has lost the right to be part of the authorities,
        /// this member will be removed from the authority set in 2 sessions.
        /// [member_id]
        MemberRemoved(T::MemberId),
    }

    // ERRORS //

    #[pallet::error]
    pub enum Error<T> {
        /// Already incoming
        AlreadyIncoming,
        /// Already online
        AlreadyOnline,
        /// Already outgoing
        AlreadyOutgoing,
        /// Not found owner key
        MemberIdNotFound,
        /// Member not found
        MemberNotFound,
        /// Neither online nor scheduled
        NotOnlineNorIncoming,
        /// Not owner
        NotOwner,
        /// Not member
        NotMember,
        /// Session keys not provided
        SessionKeysNotProvided,
        /// Too man aAuthorities
        TooManyAuthorities,
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000_000_000)]
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
        #[pallet::weight(1_000_000_000)]
        pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;
            let member_id = Self::verify_ownership_and_membership(&who)?;

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
            if AuthoritiesCounter::<T>::get() >= T::MaxAuthorities::get() {
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

        #[pallet::weight(1_000_000_000)]
        pub fn set_session_keys(
            origin: OriginFor<T>,
            keys: T::KeysWrapper,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            let member_id = Self::verify_ownership_and_membership(&who)?;

            let _post_info = pallet_session::Call::<T>::set_keys {
                keys: keys.into(),
                proof: vec![],
            }
            .dispatch_bypass_filter(origin)?;

            let current_session_index = pallet_session::Pallet::<T>::current_index();
            let expire_on_session =
                current_session_index.saturating_add(T::MaxOfflineSessions::get());
            let must_rotate_keys_before =
                current_session_index.saturating_add(T::MaxKeysLife::get());

            Members::<T>::mutate_exists(member_id, |member_data_opt| {
                let mut member_data = member_data_opt.get_or_insert(MemberData {
                    expire_on_session,
                    must_rotate_keys_before,
                    owner_key: who,
                });
                member_data.must_rotate_keys_before = must_rotate_keys_before;
            });
            MustRotateKeysBefore::<T>::append(must_rotate_keys_before, member_id);

            Ok(().into())
        }
        #[pallet::weight(1_000_000_000)]
        pub fn remove_member(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            T::RemoveMemberOrigin::ensure_origin(origin)?;

            let member_data = Members::<T>::get(member_id).ok_or(Error::<T>::NotMember)?;
            Self::do_remove_member(member_id, member_data.owner_key);

            Ok(().into())
        }
    }

    // PUBLIC FUNCTIONS //

    impl<T: Config> Pallet<T> {
        /// Should be transactional
        #[frame_support::transactional]
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
                .ok_or(Error::<T>::MemberNotFound)?;
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
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        fn do_remove_member(member_id: T::MemberId, owner_key: T::AccountId) -> Weight {
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
            Self::deposit_event(Event::MemberRemoved(member_id));
            let _ = T::OnRemovedMember::on_removed_member(member_id);

            0
        }
        pub(super) fn expire_memberships(current_session_index: SessionIndex) {
            for member_id in MembersExpireOn::<T>::take(current_session_index) {
                if let Some(member_data) = Members::<T>::get(member_id) {
                    if member_data.expire_on_session == current_session_index {
                        Self::do_remove_member(member_id, member_data.owner_key);
                    }
                }
            }
            for member_id in MustRotateKeysBefore::<T>::take(current_session_index) {
                if let Some(member_data) = Members::<T>::get(member_id) {
                    if member_data.must_rotate_keys_before == current_session_index {
                        Self::do_remove_member(member_id, member_data.owner_key);
                    }
                }
            }
        }
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
                AuthoritiesCounter::<T>::mutate(|counter| *counter += 1);
                Self::deposit_event(Event::MemberGoOnline(member_id));
            }
            not_already_inserted
        }
        fn insert_out(member_id: T::MemberId) -> bool {
            let not_already_inserted = OutgoingAuthorities::<T>::mutate(|members_ids| {
                if let Err(index) = members_ids.binary_search(&member_id) {
                    members_ids.insert(index, member_id);
                    true
                } else {
                    false
                }
            });
            if not_already_inserted {
                AuthoritiesCounter::<T>::mutate(|counter| {
                    if *counter > 0 {
                        *counter -= 1
                    }
                });
                Self::deposit_event(Event::MemberGoOffline(member_id));
            }
            not_already_inserted
        }
        fn is_incoming(member_id: T::MemberId) -> bool {
            IncomingAuthorities::<T>::get()
                .binary_search(&member_id)
                .is_ok()
        }
        fn is_online(member_id: T::MemberId) -> bool {
            OnlineAuthorities::<T>::get()
                .binary_search(&member_id)
                .is_ok()
        }
        fn is_outgoing(member_id: T::MemberId) -> bool {
            OutgoingAuthorities::<T>::get()
                .binary_search(&member_id)
                .is_ok()
        }
        fn remove_in(member_id: T::MemberId) {
            AuthoritiesCounter::<T>::mutate(|counter| counter.saturating_sub(1));
            IncomingAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            })
        }
        fn remove_online(member_id: T::MemberId) {
            AuthoritiesCounter::<T>::mutate(|counter| counter.saturating_add(1));
            OnlineAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            });
        }
        fn remove_out(member_id: T::MemberId) {
            OutgoingAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            });
        }
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
    fn new_session(session_index: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        let members_ids_to_add = IncomingAuthorities::<T>::take();
        let members_ids_to_del = OutgoingAuthorities::<T>::take();

        if members_ids_to_add.is_empty() {
            if members_ids_to_del.is_empty() {
                return None;
            } else {
                for member_id in &members_ids_to_del {
                    // Apply MaxOfflineSessions rule
                    let expire_on_session =
                        session_index.saturating_add(T::MaxOfflineSessions::get());
                    Members::<T>::mutate_exists(member_id, |member_data_opt| {
                        if let Some(ref mut member_data) = member_data_opt {
                            member_data.expire_on_session = expire_on_session;
                        }
                    });
                    MembersExpireOn::<T>::append(expire_on_session, member_id);
                }
                Self::deposit_event(Event::OutgoingAuthorities(members_ids_to_del.clone()));
            }
        } else {
            Self::deposit_event(Event::IncomingAuthorities(members_ids_to_add.clone()));
        }

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
        Self::expire_memberships(start_index);
        T::OnNewSession::on_new_session(start_index);
    }
}

fn add_full_identification<T: Config>(
    validator_id: T::ValidatorId,
) -> Option<(T::ValidatorId, T::FullIdentification)> {
    use sp_runtime::traits::Convert as _;
    T::FullIdentificationOf::convert(validator_id.clone())
        .map(|full_ident| (validator_id, full_ident))
}

impl<T: Config> pallet_session::historical::SessionManager<T::ValidatorId, T::FullIdentification>
    for Pallet<T>
{
    fn new_session(
        new_index: SessionIndex,
    ) -> Option<sp_std::vec::Vec<(T::ValidatorId, T::FullIdentification)>> {
        <Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators_ids| {
            validators_ids
                .into_iter()
                .filter_map(add_full_identification::<T>)
                .collect()
        })
    }
    fn new_session_genesis(
        new_index: SessionIndex,
    ) -> Option<sp_std::vec::Vec<(T::ValidatorId, T::FullIdentification)>> {
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
