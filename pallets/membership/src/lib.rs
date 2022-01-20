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
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use sp_membership::traits::*;
use sp_membership::{MembershipData, OriginPermission};
use sp_runtime::traits::Zero;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use std::collections::BTreeMap;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::StorageVersion;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::IsMember;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type IsIdtyAllowedToClaimMembership: IsIdtyAllowedToClaimMembership<Self::IdtyId>;
        type IsIdtyAllowedToRenewMembership: IsIdtyAllowedToRenewMembership<Self::IdtyId>;
        type IsIdtyAllowedToRequestMembership: IsIdtyAllowedToRequestMembership<Self::IdtyId>;
        type IsOriginAllowedToUseIdty: IsOriginAllowedToUseIdty<Self::Origin, Self::IdtyId>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
        /// Specify true if you want to externalize the storage of memberships, but in this case
        /// you must provide an implementation of `MembershipExternalStorage`
        type ExternalizeMembershipStorage: Get<bool>;
        /// Something that identifies an identity
        type IdtyId: Copy + MaybeSerializeDeserialize + Parameter + Ord;
        /// Optional metadata
        type MetaData: Parameter;
        /// Provide your implementation of membership storage here, if you want the pallet to
        /// handle the storage for you, specify `()` and set `ExternalizeMembershipStorage` to
        /// `false`.
        type MembershipExternalStorage: MembershipExternalStorage<Self::BlockNumber, Self::IdtyId>;
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
                if T::ExternalizeMembershipStorage::get() {
                    T::MembershipExternalStorage::insert(*idty_id, *membership_data);
                } else {
                    Membership::<T, I>::insert(idty_id, membership_data);
                }
            }
        }
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn membership)]
    pub type Membership<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::IdtyId, MembershipData<T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn memberships_expire_on)]
    pub type MembershipsExpireOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<T::IdtyId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_membership)]
    pub type PendingMembership<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::IdtyId, T::BlockNumber, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_memberships_expire_on)]
    pub type PendingMembershipsExpireOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<T::IdtyId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn revoked_membership)]
    pub type RevokedMembership<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::IdtyId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn revoked_memberships_pruned_on)]
    pub type RevokedMembershipsPrunedOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<T::IdtyId>, OptionQuery>;

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
        /// Identity not allowed to claim membership
        IdtyNotAllowedToClaimMembership,
        /// Identity not allowed to request membership
        IdtyNotAllowedToRequestMembership,
        /// Identity not allowed to renew membership
        IdtyNotAllowedToRenewMembership,
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
        pub fn request_membership(
            origin: OriginFor<T>,
            idty_id: T::IdtyId,
        ) -> DispatchResultWithPostInfo {
            let allowed =
                match T::IsOriginAllowedToUseIdty::is_origin_allowed_to_use_idty(&origin, &idty_id)
                {
                    OriginPermission::Forbidden => {
                        return Err(Error::<T, I>::OriginNotAllowedToUseIdty.into())
                    }
                    OriginPermission::Allowed => {
                        T::IsIdtyAllowedToRequestMembership::is_idty_allowed_to_request_membership(
                            &idty_id,
                        )
                    }
                    OriginPermission::Root => true,
                };
            if !allowed {
                return Err(Error::<T, I>::IdtyNotAllowedToRequestMembership.into());
            }
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

            PendingMembership::<T, I>::insert(idty_id, expire_on);
            PendingMembershipsExpireOn::<T, I>::append(expire_on, idty_id);
            Self::deposit_event(Event::MembershipRequested(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipRequested(idty_id));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn claim_membership(
            origin: OriginFor<T>,
            idty_id: T::IdtyId,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            if Membership::<T, I>::contains_key(&idty_id) {
                return Err(Error::<T, I>::MembershipAlreadyAcquired.into());
            }
            let allowed =
                match T::IsOriginAllowedToUseIdty::is_origin_allowed_to_use_idty(&origin, &idty_id)
                {
                    OriginPermission::Forbidden => {
                        return Err(Error::<T, I>::OriginNotAllowedToUseIdty.into())
                    }
                    OriginPermission::Allowed => {
                        T::IsIdtyAllowedToClaimMembership::is_idty_allowed_to_claim_membership(
                            &idty_id,
                        )
                    }
                    OriginPermission::Root => true,
                };
            if !allowed {
                return Err(Error::<T, I>::IdtyNotAllowedToClaimMembership.into());
            }

            if !PendingMembership::<T, I>::contains_key(&idty_id) {
                return Err(Error::<T, I>::MembershipRequestNotFound.into());
            }

            let _ = Self::do_claim_membership(idty_id, metadata);

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn renew_membership(
            origin: OriginFor<T>,
            idty_id: T::IdtyId,
        ) -> DispatchResultWithPostInfo {
            let allowed =
                match T::IsOriginAllowedToUseIdty::is_origin_allowed_to_use_idty(&origin, &idty_id)
                {
                    OriginPermission::Forbidden => {
                        return Err(Error::<T, I>::OriginNotAllowedToUseIdty.into())
                    }
                    OriginPermission::Allowed => {
                        T::IsIdtyAllowedToRenewMembership::is_idty_allowed_to_renew_membership(
                            &idty_id,
                        )
                    }
                    OriginPermission::Root => true,
                };
            if !allowed {
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
            idty_id: T::IdtyId,
        ) -> DispatchResultWithPostInfo {
            if T::IsOriginAllowedToUseIdty::is_origin_allowed_to_use_idty(&origin, &idty_id)
                == OriginPermission::Forbidden
            {
                return Err(Error::<T, I>::OriginNotAllowedToUseIdty.into());
            }

            let _ = Self::do_revoke_membership(idty_id);

            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        pub(super) fn do_claim_membership(idty_id: T::IdtyId, metadata: T::MetaData) -> Weight {
            let mut total_weight = 1;
            PendingMembership::<T, I>::remove(&idty_id);
            total_weight += Self::do_renew_membership_inner(idty_id);
            Self::deposit_event(Event::MembershipAcquired(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipAcquired(idty_id, metadata));
            total_weight
        }
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
        pub(super) fn do_revoke_membership(idty_id: T::IdtyId) -> Weight {
            Self::remove_membership(&idty_id);
            if T::RevocationPeriod::get() > Zero::zero() {
                let block_number = frame_system::pallet::Pallet::<T>::block_number();
                let pruned_on = block_number + T::RevocationPeriod::get();

                RevokedMembership::<T, I>::insert(idty_id, ());
                RevokedMembershipsPrunedOn::<T, I>::append(pruned_on, idty_id);
            }
            Self::deposit_event(Event::MembershipRevoked(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipRevoked(idty_id));

            0
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

            use frame_support::storage::generator::StorageMap as _;
            if let Some(identities_ids) =
                PendingMembershipsExpireOn::<T, I>::from_query_to_optional_value(
                    PendingMembershipsExpireOn::<T, I>::take(block_number),
                )
            {
                for idty_id in identities_ids {
                    PendingMembership::<T, I>::remove(&idty_id);
                    Self::deposit_event(Event::PendingMembershipExpired(idty_id));
                    total_weight += T::OnEvent::on_event(
                        &sp_membership::Event::PendingMembershipExpired(idty_id),
                    );
                }
            }

            total_weight
        }
        fn prune_revoked_memberships(block_number: T::BlockNumber) -> Weight {
            let total_weight: Weight = 0;

            use frame_support::storage::generator::StorageMap as _;
            if let Some(identities_ids) =
                RevokedMembershipsPrunedOn::<T, I>::from_query_to_optional_value(
                    RevokedMembershipsPrunedOn::<T, I>::take(block_number),
                )
            {
                for idty_id in identities_ids {
                    RevokedMembership::<T, I>::remove(idty_id);
                }
            }

            total_weight
        }

        pub(super) fn is_member_inner(idty_id: &T::IdtyId) -> bool {
            if T::ExternalizeMembershipStorage::get() {
                T::MembershipExternalStorage::is_member(idty_id)
            } else {
                Membership::<T, I>::contains_key(idty_id)
            }
        }
        fn insert_membership(idty_id: T::IdtyId, membership_data: MembershipData<T::BlockNumber>) {
            if T::ExternalizeMembershipStorage::get() {
                T::MembershipExternalStorage::insert(idty_id, membership_data);
            } else {
                Membership::<T, I>::insert(idty_id, membership_data);
            }
        }
        fn get_membership(idty_id: &T::IdtyId) -> Option<MembershipData<T::BlockNumber>> {
            if T::ExternalizeMembershipStorage::get() {
                T::MembershipExternalStorage::get(idty_id)
            } else {
                Membership::<T, I>::try_get(idty_id).ok()
            }
        }
        fn remove_membership(idty_id: &T::IdtyId) {
            if T::ExternalizeMembershipStorage::get() {
                T::MembershipExternalStorage::remove(idty_id);
            } else {
                Membership::<T, I>::remove(idty_id);
            }
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
