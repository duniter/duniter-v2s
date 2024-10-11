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

//! # Duniter Membership Pallet
//!
//! The Duniter Membership Pallet is closely integrated with the Duniter Web of Trust (WoT) and is tailored specifically for Duniter, in contrast to the [Parity Membership Pallet](https://github.com/paritytech/substrate/tree/master/frame/membership). It operates exclusively within the Duniter ecosystem and is utilized internally by the Identity, Web of Trust, and Distance Pallets.
//!
//! ## Main Web of Trust (WoT)
//!
//! The Membership Pallet manages all aspects related to the membership of identities within the Duniter Web of Trust. Unlike traditional membership systems, it does not expose any external calls to users. Instead, its functionalities are accessible through distance evaluations provided by the Distance Oracle.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use frame_support::pallet_prelude::{Weight, *};
use scale_info::prelude::{collections::BTreeMap, vec::Vec};
use sp_membership::{traits::*, MembershipData};
use sp_runtime::traits::Zero;

#[cfg(feature = "runtime-benchmarks")]
pub trait SetupBenchmark<IdtyId, AccountId> {
    fn force_valid_distance_status(idty_index: &IdtyId);
    fn add_cert(_issuer: &IdtyId, _receiver: &IdtyId);
}

#[cfg(feature = "runtime-benchmarks")]
impl<IdtyId, AccountId> SetupBenchmark<IdtyId, AccountId> for () {
    fn force_valid_distance_status(_idty_id: &IdtyId) {}

    fn add_cert(_issuer: &IdtyId, _receiver: &IdtyId) {}
}

/// Represent reasons for the removal of membership.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MembershipRemovalReason {
    /// Indicates membership was removed because it reached the end of its life.
    Expired,
    /// Indicates membership was explicitly revoked.
    Revoked,
    /// Indicates membership was removed because the received certifications count fell below the threshold.
    NotEnoughCerts,
    /// Indicates membership was removed due to system reasons (e.g., consumers, authority members, or root).
    System,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
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
        /// Check if the identity can perform membership operations.
        type CheckMembershipOpAllowed: CheckMembershipOpAllowed<Self::IdtyId>;

        /// Something that identifies an identity.
        type IdtyId: Copy + MaybeSerializeDeserialize + Parameter + Ord;

        /// Something that gives the IdtyId of an AccountId and reverse.
        type IdtyAttr: duniter_primitives::Idty<Self::IdtyId, Self::AccountId>;

        /// Maximum lifespan of a single membership (in number of blocks).
        #[pallet::constant]
        type MembershipPeriod: Get<BlockNumberFor<Self>>;

        /// Minimum delay to wait before renewing membership, i.e., asking for distance evaluation.
        #[pallet::constant]
        type MembershipRenewalPeriod: Get<BlockNumberFor<Self>>;

        /// Handler called when a new membership is created or renewed.
        type OnNewMembership: OnNewMembership<Self::IdtyId>;

        /// Handler called when a membership is revoked or removed.
        type OnRemoveMembership: OnRemoveMembership<Self::IdtyId>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet.
        type WeightInfo: WeightInfo;

        /// Benchmark setup handler for runtime benchmarks (feature-dependent).
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkSetupHandler: SetupBenchmark<Self::IdtyId, Self::AccountId>;
    }

    // GENESIS STUFF //

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub memberships: BTreeMap<T::IdtyId, MembershipData<BlockNumberFor<T>>>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                memberships: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (idty_id, membership_data) in &self.memberships {
                MembershipsExpireOn::<T>::append(membership_data.expire_on, idty_id);
                Membership::<T>::insert(idty_id, membership_data);
            }
        }
    }

    // STORAGE //

    /// The membership data for each identity.
    #[pallet::storage]
    #[pallet::getter(fn membership)]
    pub type Membership<T: Config> = CountedStorageMap<
        _,
        Twox64Concat,
        T::IdtyId,
        MembershipData<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// The identities of memberships to expire at a given block.
    #[pallet::storage]
    #[pallet::getter(fn memberships_expire_on)]
    pub type MembershipsExpireOn<T: Config> =
        StorageMap<_, Twox64Concat, BlockNumberFor<T>, Vec<T::IdtyId>, ValueQuery>;

    // EVENTS //

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A membership was added.
        MembershipAdded {
            member: T::IdtyId,
            expire_on: BlockNumberFor<T>,
        },
        /// A membership was renewed.
        MembershipRenewed {
            member: T::IdtyId,
            expire_on: BlockNumberFor<T>,
        },
        /// A membership was removed.
        MembershipRemoved {
            member: T::IdtyId,
            reason: MembershipRemovalReason,
        },
    }

    // ERRORS//

    #[pallet::error]
    pub enum Error<T> {
        /// Membership not found, can not renew.
        MembershipNotFound,
        /// Already member, can not add membership.
        AlreadyMember,
    }

    // HOOKS //

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            if n > BlockNumberFor::<T>::zero() {
                T::WeightInfo::on_initialize().saturating_add(Self::expire_memberships(n))
            } else {
                T::WeightInfo::on_initialize()
            }
        }
    }

    // // CALLS //
    // #[pallet::call]
    // impl<T: Config> Pallet<T> {
    //     // no calls for membership pallet
    // }

    // INTERNAL FUNCTIONS //
    impl<T: Config> Pallet<T> {
        /// Unschedules membership expiry.
        fn unschedule_membership_expiry(idty_id: T::IdtyId, block_number: BlockNumberFor<T>) {
            let mut scheduled = MembershipsExpireOn::<T>::get(block_number);

            if let Some(pos) = scheduled.iter().position(|x| *x == idty_id) {
                scheduled.swap_remove(pos);
                MembershipsExpireOn::<T>::set(block_number, scheduled);
            }
        }

        /// Insert membership and schedule its expiry.
        fn insert_membership_and_schedule_expiry(idty_id: T::IdtyId) -> BlockNumberFor<T> {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let expire_on = block_number + T::MembershipPeriod::get();

            Membership::<T>::insert(idty_id, MembershipData { expire_on });
            MembershipsExpireOn::<T>::append(expire_on, idty_id);
            expire_on
        }

        /// Check if membership can be claimed.
        pub fn check_add_membership(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            // no-op is error
            ensure!(
                Membership::<T>::get(idty_id).is_none(),
                Error::<T>::AlreadyMember
            );

            // check status and enough certifications
            T::CheckMembershipOpAllowed::check_add_membership(idty_id)?;
            Ok(())
        }

        /// Check if membership renewal is allowed.
        pub fn check_renew_membership(
            idty_id: T::IdtyId,
        ) -> Result<MembershipData<BlockNumberFor<T>>, DispatchError> {
            let membership_data =
                Membership::<T>::get(idty_id).ok_or(Error::<T>::MembershipNotFound)?;

            // enough certifications
            T::CheckMembershipOpAllowed::check_renew_membership(idty_id)?;
            Ok(membership_data)
        }

        /// Attempt to add membership.
        pub fn try_add_membership(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            Self::check_add_membership(idty_id)?;
            Self::do_add_membership(idty_id);
            Ok(())
        }

        /// Attempt to renew membership.
        pub fn try_renew_membership(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            let membership_data = Self::check_renew_membership(idty_id)?;
            Self::do_renew_membership(idty_id, membership_data);
            Ok(())
        }

        /// Perform membership addition.
        fn do_add_membership(idty_id: T::IdtyId) {
            let expire_on = Self::insert_membership_and_schedule_expiry(idty_id);
            Self::deposit_event(Event::MembershipAdded {
                member: idty_id,
                expire_on,
            });
            T::OnNewMembership::on_created(&idty_id);
        }

        /// Perform membership renewal.
        fn do_renew_membership(
            idty_id: T::IdtyId,
            membership_data: MembershipData<BlockNumberFor<T>>,
        ) {
            Self::unschedule_membership_expiry(idty_id, membership_data.expire_on);
            let expire_on = Self::insert_membership_and_schedule_expiry(idty_id);
            Self::deposit_event(Event::MembershipRenewed {
                member: idty_id,
                expire_on,
            });
            T::OnNewMembership::on_renewed(&idty_id);
        }

        /// Perform membership removal.
        pub fn do_remove_membership(idty_id: T::IdtyId, reason: MembershipRemovalReason) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(2, 3);
            if let Some(membership_data) = Membership::<T>::take(idty_id) {
                Self::unschedule_membership_expiry(idty_id, membership_data.expire_on);
                Self::deposit_event(Event::MembershipRemoved {
                    member: idty_id,
                    reason,
                });
                weight += T::OnRemoveMembership::on_removed(&idty_id);
            }
            weight
        }

        /// Perform membership expiry scheduled at the given block number.
        pub fn expire_memberships(block_number: BlockNumberFor<T>) -> Weight {
            let mut expired_idty_count = 0u32;

            for idty_id in MembershipsExpireOn::<T>::take(block_number) {
                // remove membership (take)
                Self::do_remove_membership(idty_id, MembershipRemovalReason::Expired);
                expired_idty_count += 1;
            }
            T::WeightInfo::expire_memberships(expired_idty_count)
        }

        /// Check if an identity is a member.
        pub fn is_member(idty_id: &T::IdtyId) -> bool {
            Membership::<T>::contains_key(idty_id)
        }
    }
}

// implement traits

impl<T: Config> sp_runtime::traits::IsMember<T::IdtyId> for Pallet<T> {
    fn is_member(idty_id: &T::IdtyId) -> bool {
        Self::is_member(idty_id)
    }
}

impl<T: Config> MembersCount for Pallet<T> {
    fn members_count() -> u32 {
        Membership::<T>::count()
    }
}
