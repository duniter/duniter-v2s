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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/*#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

pub use pallet::*;

use frame_support::dispatch::Weight;
use frame_support::error::BadOrigin;
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use sp_membership::traits::*;
use sp_membership::MembershipData;
use sp_runtime::traits::Zero;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use std::collections::BTreeMap;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Convert;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type IsIdtyAllowedToRenewMembership: IsIdtyAllowedToRenewMembership<Self::IdtyId>;
        type IsIdtyAllowedToRequestMembership: IsIdtyAllowedToRequestMembership<Self::IdtyId>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
        /// Something that identifies an identity
        type IdtyId: Copy + MaybeSerializeDeserialize + Parameter + Ord;
        /// Something that give the IdtyId on an account id
        type IdtyIdOf: Convert<Self::AccountId, Option<Self::IdtyId>>;
        /// Optional metadata
        type MetaData: Parameter + Validate<Self::AccountId>;
        #[pallet::constant]
        /// Maximum life span of a non-renewable membership (in number of blocks)
        type MembershipPeriod: Get<Self::BlockNumber>;
        /// On event handler
        type OnEvent: OnEvent<Self::IdtyId, Self::MetaData>;
        #[pallet::constant]
        /// Maximum period (in number of blocks), where an identity can remain pending subscription.
        type PendingMembershipPeriod: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Duration after which a membership is renewable
        type RenewablePeriod: Get<Self::BlockNumber>;
        #[pallet::constant]
        /// Minimum duration (in number of blocks between a revocation and a new entry request
        type RevocationPeriod: Get<Self::BlockNumber>;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub memberships: BTreeMap<T::IdtyId, MembershipData<T::BlockNumber>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                memberships: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            for (idty_id, membership_data) in &self.memberships {
                MembershipsExpireOn::<T, I>::append(membership_data.expire_on, idty_id);
                Membership::<T, I>::insert(idty_id, membership_data);
            }
        }
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn membership)]
    pub type Membership<T: Config<I>, I: 'static = ()> =
        CountedStorageMap<_, Twox64Concat, T::IdtyId, MembershipData<T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn memberships_expire_on)]
    pub type MembershipsExpireOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::IdtyId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_membership)]
    pub type PendingMembership<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::IdtyId, T::MetaData, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_memberships_expire_on)]
    pub type PendingMembershipsExpireOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::IdtyId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn revoked_membership)]
    pub type RevokedMembership<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::IdtyId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn revoked_memberships_pruned_on)]
    pub type RevokedMembershipsPrunedOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::IdtyId>, OptionQuery>;

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// A membership has acquired
        /// [idty_id]
        MembershipAcquired(T::IdtyId),
        /// A membership has expired
        /// [idty_id]
        MembershipExpired(T::IdtyId),
        /// A membership has renewed
        /// [idty_id]
        MembershipRenewed(T::IdtyId),
        /// An identity requested membership
        /// [idty_id]
        MembershipRequested(T::IdtyId),
        /// A membership has revoked
        /// [idty_id]
        MembershipRevoked(T::IdtyId),
        /// A pending membership request has expired
        /// [idty_id]
        PendingMembershipExpired(T::IdtyId),
    }

    // ERRORS//

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Identity not allowed to request membership
        IdtyNotAllowedToRequestMembership,
        /// Identity not allowed to renew membership
        IdtyNotAllowedToRenewMembership,
        /// Invalid meta data
        InvalidMetaData,
        /// Identity id not found
        IdtyIdNotFound,
        /// Membership already acquired
        MembershipAlreadyAcquired,
        /// Membership already requested
        MembershipAlreadyRequested,
        /// Membership not yet renewable
        MembershipNotYetRenewable,
        /// Membership not found
        MembershipNotFound,
        /// Origin not allowed to use this identity
        OriginNotAllowedToUseIdty,
        /// Membership request not found
        MembershipRequestNotFound,
        /// Membership revoked recently
        MembershipRevokedRecently,
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if n > T::BlockNumber::zero() {
                Self::expire_pending_memberships(n)
                    + Self::expire_memberships(n)
                    + Self::prune_revoked_memberships(n)
            } else {
                0
            }
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        #[pallet::weight(0)]
        pub fn force_request_membership(
            origin: OriginFor<T>,
            idty_id: T::IdtyId,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Self::do_request_membership(idty_id, metadata)
        }

        #[pallet::weight(0)]
        pub fn request_membership(
            origin: OriginFor<T>,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let idty_id = T::IdtyIdOf::convert(who.clone()).ok_or(Error::<T, I>::IdtyIdNotFound)?;
            if !metadata.validate(&who) {
                return Err(Error::<T, I>::InvalidMetaData.into());
            }
            if !T::IsIdtyAllowedToRequestMembership::is_idty_allowed_to_request_membership(&idty_id)
            {
                return Err(Error::<T, I>::IdtyNotAllowedToRequestMembership.into());
            }

            Self::do_request_membership(idty_id, metadata)
        }

        #[pallet::weight(0)]
        pub fn claim_membership(
            origin: OriginFor<T>,
            maybe_idty_id: Option<T::IdtyId>,
        ) -> DispatchResultWithPostInfo {
            // Verify phase
            let idty_id = Self::ensure_origin_and_get_idty_id(origin, maybe_idty_id)?;

            if Membership::<T, I>::contains_key(&idty_id) {
                return Err(Error::<T, I>::MembershipAlreadyAcquired.into());
            }

            let metadata = PendingMembership::<T, I>::take(&idty_id)
                .ok_or(Error::<T, I>::MembershipRequestNotFound)?;

            // Apply phase
            Self::do_renew_membership_inner(idty_id);
            Self::deposit_event(Event::MembershipAcquired(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipAcquired(idty_id, metadata));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn renew_membership(
            origin: OriginFor<T>,
            maybe_idty_id: Option<T::IdtyId>,
        ) -> DispatchResultWithPostInfo {
            // Verify phase
            let idty_id = Self::ensure_origin_and_get_idty_id(origin, maybe_idty_id)?;

            if !T::IsIdtyAllowedToRenewMembership::is_idty_allowed_to_renew_membership(&idty_id) {
                return Err(Error::<T, I>::IdtyNotAllowedToRenewMembership.into());
            }

            if let Some(membership_data) = Self::get_membership(&idty_id) {
                let block_number = frame_system::pallet::Pallet::<T>::block_number();
                if membership_data.renewable_on > block_number {
                    return Err(Error::<T, I>::MembershipNotYetRenewable.into());
                }
            } else {
                return Err(Error::<T, I>::MembershipNotFound.into());
            }

            let _ = Self::do_renew_membership(idty_id);

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn revoke_membership(
            origin: OriginFor<T>,
            maybe_idty_id: Option<T::IdtyId>,
        ) -> DispatchResultWithPostInfo {
            // Verify phase
            let idty_id = Self::ensure_origin_and_get_idty_id(origin, maybe_idty_id)?;

            // Apply phase
            let _ = Self::do_revoke_membership(idty_id);

            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        pub(super) fn do_renew_membership(idty_id: T::IdtyId) -> Weight {
            let total_weight = Self::do_renew_membership_inner(idty_id);
            Self::deposit_event(Event::MembershipRenewed(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipRenewed(idty_id));
            total_weight
        }
        fn do_renew_membership_inner(idty_id: T::IdtyId) -> Weight {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let expire_on = block_number + T::MembershipPeriod::get();
            let renewable_on = block_number + T::RenewablePeriod::get();

            Self::insert_membership(
                idty_id,
                MembershipData {
                    expire_on,
                    renewable_on,
                },
            );
            MembershipsExpireOn::<T, I>::append(expire_on, idty_id);
            0
        }
        fn do_request_membership(
            idty_id: T::IdtyId,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            if PendingMembership::<T, I>::contains_key(&idty_id) {
                return Err(Error::<T, I>::MembershipAlreadyRequested.into());
            }
            if Membership::<T, I>::contains_key(&idty_id) {
                return Err(Error::<T, I>::MembershipAlreadyAcquired.into());
            }
            if RevokedMembership::<T, I>::contains_key(&idty_id) {
                return Err(Error::<T, I>::MembershipRevokedRecently.into());
            }

            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let expire_on = block_number + T::PendingMembershipPeriod::get();

            PendingMembership::<T, I>::insert(idty_id, metadata);
            PendingMembershipsExpireOn::<T, I>::append(expire_on, idty_id);
            Self::deposit_event(Event::MembershipRequested(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipRequested(idty_id));

            Ok(().into())
        }
        pub(super) fn do_revoke_membership(idty_id: T::IdtyId) -> Weight {
            if Self::remove_membership(&idty_id) {
                if T::RevocationPeriod::get() > Zero::zero() {
                    let block_number = frame_system::pallet::Pallet::<T>::block_number();
                    let pruned_on = block_number + T::RevocationPeriod::get();

                    RevokedMembership::<T, I>::insert(idty_id, ());
                    RevokedMembershipsPrunedOn::<T, I>::append(pruned_on, idty_id);
                }
                Self::deposit_event(Event::MembershipRevoked(idty_id));
                T::OnEvent::on_event(&sp_membership::Event::MembershipRevoked(idty_id));
            }

            0
        }
        fn ensure_origin_and_get_idty_id(
            origin: OriginFor<T>,
            maybe_idty_id: Option<T::IdtyId>,
        ) -> Result<T::IdtyId, DispatchError> {
            match origin.into() {
                Ok(RawOrigin::Root) => {
                    maybe_idty_id.ok_or_else(|| Error::<T, I>::IdtyIdNotFound.into())
                }
                Ok(RawOrigin::Signed(account_id)) => T::IdtyIdOf::convert(account_id)
                    .ok_or_else(|| Error::<T, I>::IdtyIdNotFound.into()),
                _ => Err(BadOrigin.into()),
            }
        }
        fn expire_memberships(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0;

            for idty_id in MembershipsExpireOn::<T, I>::take(block_number) {
                if let Some(member_data) = Self::get_membership(&idty_id) {
                    if member_data.expire_on == block_number {
                        Self::remove_membership(&idty_id);
                        Self::deposit_event(Event::MembershipExpired(idty_id));
                        total_weight +=
                            T::OnEvent::on_event(&sp_membership::Event::MembershipExpired(idty_id));
                    }
                }
            }

            total_weight
        }
        fn expire_pending_memberships(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0;

            for idty_id in PendingMembershipsExpireOn::<T, I>::take(block_number) {
                PendingMembership::<T, I>::remove(&idty_id);
                Self::deposit_event(Event::PendingMembershipExpired(idty_id));
                total_weight +=
                    T::OnEvent::on_event(&sp_membership::Event::PendingMembershipExpired(idty_id));
            }

            total_weight
        }
        fn prune_revoked_memberships(block_number: T::BlockNumber) -> Weight {
            let total_weight: Weight = 0;

            if let Some(identities_ids) = RevokedMembershipsPrunedOn::<T, I>::take(block_number) {
                for idty_id in identities_ids {
                    RevokedMembership::<T, I>::remove(idty_id);
                }
            }

            total_weight
        }

        pub(super) fn is_member_inner(idty_id: &T::IdtyId) -> bool {
            Membership::<T, I>::contains_key(idty_id)
        }
        fn insert_membership(idty_id: T::IdtyId, membership_data: MembershipData<T::BlockNumber>) {
            Membership::<T, I>::insert(idty_id, membership_data);
        }
        fn get_membership(idty_id: &T::IdtyId) -> Option<MembershipData<T::BlockNumber>> {
            Membership::<T, I>::try_get(idty_id).ok()
        }
        fn remove_membership(idty_id: &T::IdtyId) -> bool {
            Membership::<T, I>::take(idty_id).is_some()
        }
    }
}

impl<T: Config<I>, I: 'static> IsInPendingMemberships<T::IdtyId> for Pallet<T, I> {
    fn is_in_pending_memberships(idty_id: T::IdtyId) -> bool {
        PendingMembership::<T, I>::contains_key(idty_id)
    }
}

impl<T: Config<I>, I: 'static> sp_runtime::traits::IsMember<T::IdtyId> for Pallet<T, I> {
    fn is_member(idty_id: &T::IdtyId) -> bool {
        Self::is_member_inner(idty_id)
    }
}

impl<T: Config<I>, I: 'static> MembersCount for Pallet<T, I> {
    fn members_count() -> u32 {
        Membership::<T, I>::count()
    }
}
