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

use frame_support::pallet_prelude::Weight;
use frame_support::pallet_prelude::*;
use sp_membership::traits::*;
use sp_membership::MembershipData;
use sp_runtime::traits::Zero;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

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

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MembershipRemovalReason {
    // reach end of life
    Expired,
    // was explicitly revoked
    Revoked,
    // received certs count passed below threshold
    NotEnoughCerts,
    // system reasons (consumers, authority members, or root)
    System,
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
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Ask the runtime whether the identity can perform membership operations
        type CheckMembershipOpAllowed: CheckMembershipOpAllowed<Self::IdtyId>;
        /// Something that identifies an identity
        type IdtyId: Copy + MaybeSerializeDeserialize + Parameter + Ord;
        /// Something that gives the IdtyId of an AccountId
        type IdtyIdOf: Convert<Self::AccountId, Option<Self::IdtyId>>;
        /// Something that gives the AccountId of an IdtyId
        type AccountIdOf: Convert<Self::IdtyId, Option<Self::AccountId>>;
        /// Maximum life span of a single membership (in number of blocks)
        // (this could be renamed "validity" or "duration")
        #[pallet::constant]
        type MembershipPeriod: Get<BlockNumberFor<Self>>;
        /// Minimum delay to wait before renewing membership
        // i.e. asking for distance evaluation
        #[pallet::constant]
        type MembershipRenewalPeriod: Get<BlockNumberFor<Self>>;
        /// On new and renew membership handler.
        type OnNewMembership: OnNewMembership<Self::IdtyId>;
        /// On revoked and removed membership handler.
        type OnRemoveMembership: OnRemoveMembership<Self::IdtyId>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
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

    /// maps identity id to membership data
    // (expiration block for instance)
    #[pallet::storage]
    #[pallet::getter(fn membership)]
    pub type Membership<T: Config> = CountedStorageMap<
        _,
        Twox64Concat,
        T::IdtyId,
        MembershipData<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// maps block number to the list of identity id set to expire at this block
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
        /// unschedule membership expiry
        fn unschedule_membership_expiry(idty_id: T::IdtyId, block_number: BlockNumberFor<T>) {
            let mut scheduled = MembershipsExpireOn::<T>::get(block_number);

            if let Some(pos) = scheduled.iter().position(|x| *x == idty_id) {
                scheduled.swap_remove(pos);
                MembershipsExpireOn::<T>::set(block_number, scheduled);
            }
        }

        /// schedule membership expiry
        fn insert_membership_and_schedule_expiry(idty_id: T::IdtyId) -> BlockNumberFor<T> {
            let block_number = frame_system::pallet::Pallet::<T>::block_number();
            let expire_on = block_number + T::MembershipPeriod::get();

            Membership::<T>::insert(idty_id, MembershipData { expire_on });
            MembershipsExpireOn::<T>::append(expire_on, idty_id);
            expire_on
        }

        /// check that membership can be claimed
        pub fn check_add_membership(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            // no-op is error
            ensure!(
                Membership::<T>::get(idty_id).is_none(),
                Error::<T>::AlreadyMember
            );

            // enough certifications and distance rule for example
            T::CheckMembershipOpAllowed::check_add_membership(idty_id)?;
            Ok(())
        }

        /// check that membership can be renewed
        pub fn check_renew_membership(
            idty_id: T::IdtyId,
        ) -> Result<MembershipData<BlockNumberFor<T>>, DispatchError> {
            let membership_data =
                Membership::<T>::get(idty_id).ok_or(Error::<T>::MembershipNotFound)?;

            // enough certifications
            T::CheckMembershipOpAllowed::check_renew_membership(idty_id)?;
            Ok(membership_data)
        }

        /// try claim membership
        pub fn try_add_membership(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            Self::check_add_membership(idty_id)?;
            Self::do_add_membership(idty_id);
            Ok(())
        }

        /// try renew membership
        pub fn try_renew_membership(idty_id: T::IdtyId) -> Result<(), DispatchError> {
            let membership_data = Self::check_renew_membership(idty_id)?;
            Self::do_renew_membership(idty_id, membership_data);
            Ok(())
        }

        /// perform membership addition
        fn do_add_membership(idty_id: T::IdtyId) {
            let expire_on = Self::insert_membership_and_schedule_expiry(idty_id);
            Self::deposit_event(Event::MembershipAdded {
                member: idty_id,
                expire_on,
            });
            T::OnNewMembership::on_created(&idty_id);
        }

        /// perform membership renewal
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

        /// perform membership removal
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

        /// perform the membership expiry scheduled at given block
        pub fn expire_memberships(block_number: BlockNumberFor<T>) -> Weight {
            let mut expired_idty_count = 0u32;

            for idty_id in MembershipsExpireOn::<T>::take(block_number) {
                // remove membership (take)
                Self::do_remove_membership(idty_id, MembershipRemovalReason::Expired);
                expired_idty_count += 1;
            }
            T::WeightInfo::expire_memberships(expired_idty_count)
        }

        /// check if identity is member
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
