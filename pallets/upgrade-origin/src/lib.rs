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
#![allow(clippy::boxed_local)]

use frame_support::{
    dispatch::PostDispatchInfo,
    traits::{IsSubType, UnfilteredDispatchable},
    weights::GetDispatchInfo,
};
use sp_runtime::traits::Dispatchable;
use sp_std::prelude::*;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + UnfilteredDispatchable<Origin = Self::Origin>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::Call>;

        /// The upgradable origin
        type UpgradableOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event {
        /// A call was dispatched as root from an upgradable origin
        DispatchedAsRoot { result: DispatchResult },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Dispatches a function call from root origin.
        ///
        /// The weight of this call is defined by the caller.
        #[pallet::weight(*_weight)]
        pub fn dispatch_as_root(
            origin: OriginFor<T>,
            call: Box<<T as Config>::Call>,
            _weight: Weight,
        ) -> DispatchResultWithPostInfo {
            T::UpgradableOrigin::ensure_origin(origin)?;

            let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());

            Self::deposit_event(Event::DispatchedAsRoot {
                result: res.map(|_| ()).map_err(|e| e.error),
            });
            Ok(Pays::No.into())
        }
    }
}
