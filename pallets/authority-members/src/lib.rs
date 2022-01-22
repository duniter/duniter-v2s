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

use frame_support::traits::Get;
use sp_staking::SessionIndex;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::traits::OnRemovedMember;
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
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_session::Config + pallet_session::historical::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type KeysWrapper: Parameter + Into<Self::Keys>;
        type IsMember: IsMember<Self::MemberId>;
        type OnRemovedMember: OnRemovedMember<Self::MemberId>;
        type OwnerKeyOf: Convert<Self::MemberId, Option<Self::AccountId>>;
        type MemberId: Copy + MaybeSerializeDeserialize + Parameter + Ord;
        #[pallet::constant]
        type MaxOfflineSessions: Get<SessionIndex>;
        type RefreshValidatorIdOrigin: EnsureOrigin<Self::Origin>;
        type RemoveMemberOrigin: EnsureOrigin<Self::Origin>;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_authorities: BTreeMap<T::MemberId, (T::ValidatorId, bool)>,
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
            for (member_id, (validator_id, _is_online)) in &self.initial_authorities {
                Members::<T>::insert(member_id, MemberData::new_genesis(validator_id.clone()));
            }
            let mut members_ids = self
                .initial_authorities
                .iter()
                .filter_map(
                    |(member_id, (_validator_id, is_online))| {
                        if *is_online {
                            Some(*member_id)
                        } else {
                            None
                        }
                    },
                )
                .collect::<Vec<T::MemberId>>();
            members_ids.sort();

            OnlineAuthorities::<T>::put(members_ids);
        }
    }

    // STORAGE //

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
        StorageMap<_, Blake2_128Concat, T::MemberId, MemberData<T::ValidatorId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn members_expire_on)]
    pub type MembersExpireOn<T: Config> =
        StorageMap<_, Blake2_128Concat, SessionIndex, Vec<T::MemberId>, ValueQuery>;

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
        /// A member has lost the right to be part of the authorities, he will be removed from
        //// the authority set in 2 sessions.
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
        OwnerKeyNotFound,
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
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn go_offline(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;
            Self::verify_ownership_and_membership(&who, member_id)?;

            let member_data = Members::<T>::get(member_id).ok_or(Error::<T>::MemberNotFound)?;
            if !pallet_session::Pallet::<T>::is_registered(&member_data.validator_id) {
                return Err(Error::<T>::SessionKeysNotProvided.into());
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
        #[pallet::weight(0)]
        pub fn go_online(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            // Verification phase //
            let who = ensure_signed(origin)?;
            Self::verify_ownership_and_membership(&who, member_id)?;

            let member_data = Members::<T>::get(member_id).ok_or(Error::<T>::MemberNotFound)?;
            if !pallet_session::Pallet::<T>::is_registered(&member_data.validator_id) {
                return Err(Error::<T>::SessionKeysNotProvided.into());
            }

            if Self::is_incoming(member_id) {
                return Err(Error::<T>::AlreadyIncoming.into());
            }
            let is_outgoing = Self::is_outgoing(member_id);
            if Self::is_online(member_id) && !is_outgoing {
                return Err(Error::<T>::AlreadyOnline.into());
            }

            // Apply phase //
            if is_outgoing {
                Self::remove_out(member_id);
            } else {
                Self::insert_in(member_id);
            }

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn set_session_keys(
            origin: OriginFor<T>,
            member_id: T::MemberId,
            keys: T::KeysWrapper,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            Self::verify_ownership_and_membership(&who, member_id)?;

            let validator_id = T::ValidatorIdOf::convert(who)
                .ok_or(pallet_session::Error::<T>::NoAssociatedValidatorId)?;

            let _post_info = pallet_session::Call::<T>::set_keys {
                keys: keys.into(),
                proof: vec![],
            }
            .dispatch_bypass_filter(origin)?;

            let expire_on_session = pallet_session::Pallet::<T>::current_index()
                .saturating_add(T::MaxOfflineSessions::get());

            Members::<T>::mutate_exists(member_id, |member_data_opt| {
                let mut member_data = member_data_opt.get_or_insert(MemberData {
                    expire_on_session,
                    validator_id: validator_id.clone(),
                });
                member_data.validator_id = validator_id;
            });

            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn refresh_validator_id(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            T::RefreshValidatorIdOrigin::ensure_origin(origin)?;

            let owner = T::OwnerKeyOf::convert(member_id).ok_or(Error::<T>::OwnerKeyNotFound)?;
            let validator_id = T::ValidatorIdOf::convert(owner)
                .ok_or(pallet_session::Error::<T>::NoAssociatedValidatorId)?;

            if !T::IsMember::is_member(&member_id) {
                return Err(Error::<T>::NotMember.into());
            }

            let expire_on_session = pallet_session::Pallet::<T>::current_index()
                .saturating_add(T::MaxOfflineSessions::get());

            Members::<T>::mutate(member_id, |member_data_opt| {
                let validator_id_clone = validator_id.clone();
                member_data_opt
                    .get_or_insert(MemberData::new(validator_id, expire_on_session))
                    .validator_id = validator_id_clone;
            });

            Ok(().into())
        }
        #[pallet::weight(0)]
        pub fn remove_member(
            origin: OriginFor<T>,
            member_id: T::MemberId,
        ) -> DispatchResultWithPostInfo {
            T::RemoveMemberOrigin::ensure_origin(origin)?;

            if !T::IsMember::is_member(&member_id) {
                return Err(Error::<T>::NotMember.into());
            }

            if let Some(owner) = T::OwnerKeyOf::convert(member_id) {
                let _post_info = pallet_session::Call::<T>::purge_keys {}
                    .dispatch_bypass_filter(frame_system::Origin::<T>::Signed(owner).into())?;
            }

            Self::do_remove_member(member_id);

            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config> Pallet<T> {
        fn do_remove_member(member_id: T::MemberId) -> Weight {
            if Self::is_online(member_id) {
                // Trigger the member deletion for next session
                Self::insert_out(member_id);
            }

            // remove all member data
            Self::remove_in(member_id);
            Self::remove_online(member_id);
            Members::<T>::remove(member_id);

            Self::deposit_event(Event::MemberRemoved(member_id));
            let _ = T::OnRemovedMember::on_removed_member(member_id);

            0
        }
        pub(super) fn expire_memberships(current_session_index: SessionIndex) {
            for member_id in MembersExpireOn::<T>::take(current_session_index) {
                if let Some(member_data) = Members::<T>::get(member_id) {
                    if member_data.expire_on_session == current_session_index {
                        Self::do_remove_member(member_id);
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
            IncomingAuthorities::<T>::mutate(|members_ids| {
                if let Ok(index) = members_ids.binary_search(&member_id) {
                    members_ids.remove(index);
                }
            })
        }
        fn remove_online(member_id: T::MemberId) {
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
            member_id: T::MemberId,
        ) -> Result<(), DispatchError> {
            if let Some(owner) = T::OwnerKeyOf::convert(member_id) {
                if who != &owner {
                    return Err(Error::<T>::NotOwner.into());
                }
            } else {
                return Err(Error::<T>::OwnerKeyNotFound.into());
            }

            if !T::IsMember::is_member(&member_id) {
                return Err(Error::<T>::NotMember.into());
            }

            Ok(())
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
                // Apply MaxOfflineSessions rule
                for member_id in &members_ids_to_del {
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
            .iter()
            .filter_map(Members::<T>::get)
            .map(|member_data| member_data.validator_id)
            .collect(),
        )
    }
    /// Same as `new_session`, but it this should only be called at genesis.
    fn new_session_genesis(_new_index: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        Some(
            OnlineAuthorities::<T>::get()
                .iter()
                .filter_map(Members::<T>::get)
                .map(|member_data| member_data.validator_id)
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
