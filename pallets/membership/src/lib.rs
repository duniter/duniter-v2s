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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

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

#[cfg(feature = "runtime-benchmarks")]
pub trait SetupBenchmark<IdtyId, AccountId> {
    fn force_status_ok(idty_index: &IdtyId, account: &AccountId) -> ();
    fn add_cert(_issuer: &IdtyId, _receiver: &IdtyId) -> ();
}

#[cfg(feature = "runtime-benchmarks")]
impl<IdtyId, AccountId> SetupBenchmark<IdtyId, AccountId> for () {
    fn force_status_ok(_idty_id: &IdtyId, _account: &AccountId) -> () {}
    fn add_cert(_issuer: &IdtyId, _receiver: &IdtyId) -> () {}
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Convert;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// Ask the runtime whether the identity can perform membership operations
        type CheckMembershipCallAllowed: CheckMembershipCallAllowed<Self::IdtyId>;
        /// Something that identifies an identity
        type IdtyId: Copy + MaybeSerializeDeserialize + Parameter + Ord;
        /// Something that gives the IdtyId of an AccountId
        type IdtyIdOf: Convert<Self::AccountId, Option<Self::IdtyId>>;
        /// Something that gives the AccountId of an IdtyId
        type AccountIdOf: Convert<Self::IdtyId, Option<Self::AccountId>>;
        /// Optional metadata
        type MetaData: Default + Parameter + Validate<Self::AccountId>;
        #[pallet::constant]
        /// Maximum life span of a non-renewable membership (in number of blocks)
        type MembershipPeriod: Get<Self::BlockNumber>;
        /// On event handler
        type OnEvent: OnEvent<Self::IdtyId, Self::MetaData>;
        #[pallet::constant]
        /// Maximum period (in number of blocks), where an identity can remain pending subscription.
        type PendingMembershipPeriod: Get<Self::BlockNumber>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkSetupHandler: SetupBenchmark<Self::IdtyId, Self::AccountId>;
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

    /// maps identity id to membership data
    // (expiration block for instance)
    #[pallet::storage]
    #[pallet::getter(fn membership)]
    pub type Membership<T: Config<I>, I: 'static = ()> =
        CountedStorageMap<_, Twox64Concat, T::IdtyId, MembershipData<T::BlockNumber>, OptionQuery>;

    /// maps block number to the list of identity id set to expire at this block
    #[pallet::storage]
    #[pallet::getter(fn memberships_expire_on)]
    pub type MembershipsExpireOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::IdtyId>, ValueQuery>;

    /// maps identity id to pending membership metadata
    #[pallet::storage]
    #[pallet::getter(fn pending_membership)]
    pub type PendingMembership<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::IdtyId, T::MetaData, OptionQuery>;

    /// maps block number to the list of memberships set to expire at this block
    #[pallet::storage]
    #[pallet::getter(fn pending_memberships_expire_on)]
    pub type PendingMembershipsExpireOn<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::IdtyId>, ValueQuery>;

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// A membership was acquired
        /// [idty_id]
        MembershipAcquired(T::IdtyId),
        /// A membership expired
        /// [idty_id]
        MembershipExpired(T::IdtyId),
        /// A membership was renewed
        /// [idty_id]
        MembershipRenewed(T::IdtyId),
        /// An membership was requested
        /// [idty_id]
        MembershipRequested(T::IdtyId),
        /// A membership was revoked
        /// [idty_id]
        MembershipRevoked(T::IdtyId),
        /// A pending membership request has expired
        /// [idty_id]
        PendingMembershipExpired(T::IdtyId),
    }

    // ERRORS//

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Invalid meta data
        InvalidMetaData,
        /// Identity id not found
        IdtyIdNotFound,
        /// Membership already acquired
        MembershipAlreadyAcquired,
        /// Membership already requested
        MembershipAlreadyRequested,
        /// Membership not found
        MembershipNotFound,
        /// Origin not allowed to use this identity
        OriginNotAllowedToUseIdty,
        /// Membership request not found
        MembershipRequestNotFound,
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if n > T::BlockNumber::zero() {
                Self::expire_pending_memberships(n) + Self::expire_memberships(n)
            } else {
                Weight::zero()
            }
        }
    }

    // CALLS //

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// submit a membership request (must have a declared identity)
        /// (only available for sub wot, automatic for main wot)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::request_membership())]
        pub fn request_membership(
            origin: OriginFor<T>,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let idty_id = T::IdtyIdOf::convert(who.clone()).ok_or(Error::<T, I>::IdtyIdNotFound)?;
            if !metadata.validate(&who) {
                return Err(Error::<T, I>::InvalidMetaData.into());
            }
            T::CheckMembershipCallAllowed::check_idty_allowed_to_request_membership(&idty_id)?;

            Self::do_request_membership(idty_id, metadata)
        }

        /// claim membership  
        /// a pending membership should exist  
        /// it must fullfill the requirements (certs, distance)  
        /// for main wot claim_membership is called automatically when validating identity  
        /// for smith wot, it means joining the authority members  
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim_membership())]
        pub fn claim_membership(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // get identity
            let idty_id = Self::get_idty_id(origin)?;

            Self::check_allowed_to_claim(idty_id)?;
            Self::do_claim_membership(idty_id);
            Ok(().into())
        }

        /// extend the validity period of an active membership
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::renew_membership())]
        pub fn renew_membership(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Verify phase
            let idty_id = Self::get_idty_id(origin)?;
            let membership_data =
                Membership::<T, I>::get(idty_id).ok_or(Error::<T, I>::MembershipNotFound)?;

            T::CheckMembershipCallAllowed::check_idty_allowed_to_renew_membership(&idty_id)?;

            // apply phase
            Self::unschedule_membership_expiry(idty_id, membership_data.expire_on);
            Self::insert_membership_and_schedule_expiry(idty_id);
            Self::deposit_event(Event::MembershipRenewed(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipRenewed(idty_id));

            Ok(().into())
        }

        /// revoke an active membership
        /// (only available for sub wot, automatic for main wot)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::revoke_membership())]
        pub fn revoke_membership(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Verify phase
            let idty_id = Self::get_idty_id(origin)?;

            // Apply phase
            Self::do_revoke_membership(idty_id);

            Ok(().into())
        }
    }

    // INTERNAL FUNCTIONS //

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// force request membership
        pub fn force_request_membership(
            idty_id: T::IdtyId,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            Self::do_request_membership(idty_id, metadata)
        }

        /// force expire membership
        pub fn force_expire_membership(idty_id: T::IdtyId) -> DispatchResultWithPostInfo {
            let new_expire_on = frame_system::pallet::Pallet::<T>::block_number()
                + T::PendingMembershipPeriod::get();
            Self::do_expire_membership(idty_id, new_expire_on);

            Ok(().into())
        }

        /// try to claim membership if not already done
        pub fn try_claim_membership(idty_id: T::IdtyId) {
            if Self::check_allowed_to_claim(idty_id).is_ok() {
                Self::do_claim_membership(idty_id);
            }
            // else { should not try to claim membership if not allowed}
        }

        /// force revoke membership
        pub fn force_revoke_membership(idty_id: T::IdtyId) {
            Self::do_revoke_membership(idty_id);
        }

        /// unschedule membership expiry
        fn unschedule_membership_expiry(idty_id: T::IdtyId, block_number: T::BlockNumber) {
            let mut scheduled = MembershipsExpireOn::<T, I>::get(block_number);

            if let Some(pos) = scheduled.iter().position(|x| *x == idty_id) {
                scheduled.swap_remove(pos);
                MembershipsExpireOn::<T, I>::set(block_number, scheduled);
            }
        }
        /// schedule membership expiry
        fn insert_membership_and_schedule_expiry(idty_id: T::IdtyId) {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let expire_on = block_number + T::MembershipPeriod::get();

            Membership::<T, I>::insert(idty_id, MembershipData { expire_on });
            MembershipsExpireOn::<T, I>::append(expire_on, idty_id);
        }

        /// perform the membership request
        fn do_request_membership(
            idty_id: T::IdtyId,
            metadata: T::MetaData,
        ) -> DispatchResultWithPostInfo {
            // checks
            if PendingMembership::<T, I>::contains_key(idty_id) {
                return Err(Error::<T, I>::MembershipAlreadyRequested.into());
            }
            if Membership::<T, I>::contains_key(idty_id) {
                return Err(Error::<T, I>::MembershipAlreadyAcquired.into());
            }

            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let expire_on = block_number + T::PendingMembershipPeriod::get();

            // apply membership request
            PendingMembership::<T, I>::insert(idty_id, metadata);
            PendingMembershipsExpireOn::<T, I>::append(expire_on, idty_id);
            Self::deposit_event(Event::MembershipRequested(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipRequested(idty_id));

            Ok(().into())
        }

        /// check that membership can be claimed
        pub fn check_allowed_to_claim(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            PendingMembership::<T, I>::get(idty_id)
                .ok_or(Error::<T, I>::MembershipRequestNotFound)?;
            // enough certifications and distance rule for example
            T::CheckMembershipCallAllowed::check_idty_allowed_to_claim_membership(&idty_id)?;
            Ok(())
        }

        /// perform membership claim
        fn do_claim_membership(idty_id: T::IdtyId) {
            if let Some(metadata) = PendingMembership::<T, I>::take(idty_id) {
                Self::insert_membership_and_schedule_expiry(idty_id);
                Self::deposit_event(Event::MembershipAcquired(idty_id));
                T::OnEvent::on_event(&sp_membership::Event::MembershipAcquired(idty_id, metadata));
            }
            // else { unreachable if check_allowed_to_claim called before }
        }

        /// perform membership revokation
        fn do_revoke_membership(idty_id: T::IdtyId) {
            if let Some(membership_data) = Membership::<T, I>::take(idty_id) {
                Self::unschedule_membership_expiry(idty_id, membership_data.expire_on);
                Self::deposit_event(Event::MembershipRevoked(idty_id));
                T::OnEvent::on_event(&sp_membership::Event::MembershipRevoked(idty_id));
            }
        }

        /// perform mebership expiration
        // add pending membership and schedule expiry of pending membership
        fn do_expire_membership(idty_id: T::IdtyId, expire_on: T::BlockNumber) -> Weight {
            if Membership::<T, I>::take(idty_id).is_some() {
                PendingMembership::<T, I>::insert(idty_id, T::MetaData::default());
                PendingMembershipsExpireOn::<T, I>::append(expire_on, idty_id);
            } // else should not happen

            Self::deposit_event(Event::MembershipExpired(idty_id));
            T::OnEvent::on_event(&sp_membership::Event::MembershipExpired(idty_id))
        }

        /// check the origin and get identity id if valid
        fn get_idty_id(origin: OriginFor<T>) -> Result<T::IdtyId, DispatchError> {
            if let Ok(RawOrigin::Signed(account_id)) = origin.into() {
                T::IdtyIdOf::convert(account_id).ok_or_else(|| Error::<T, I>::IdtyIdNotFound.into())
            } else {
                Err(BadOrigin.into())
            }
        }
        /// perform the membership expiry scheduled at given block
        // MembershipExpired events should be handeled by main wot and delete identity
        // expired membership get back to pending membership
        fn expire_memberships(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = Weight::zero();
            let new_expire_on = block_number + T::PendingMembershipPeriod::get();

            for idty_id in MembershipsExpireOn::<T, I>::take(block_number) {
                // remove membership (take)
                total_weight += Self::do_expire_membership(idty_id, new_expire_on);
            }

            total_weight
        }
        /// perform the expiration of pending membership planned at given block
        // only expire pending membership if still pending
        fn expire_pending_memberships(block_number: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = Weight::zero();

            for idty_id in PendingMembershipsExpireOn::<T, I>::take(block_number) {
                if PendingMembership::<T, I>::take(idty_id).is_some() {
                    Self::deposit_event(Event::PendingMembershipExpired(idty_id));
                    total_weight += T::OnEvent::on_event(
                        &sp_membership::Event::PendingMembershipExpired(idty_id),
                    );
                }
            }

            total_weight
        }

        /// check if identity is member
        pub(super) fn is_member_inner(idty_id: &T::IdtyId) -> bool {
            Membership::<T, I>::contains_key(idty_id)
        }
    }
}

// implement traits

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
