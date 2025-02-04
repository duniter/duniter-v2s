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
#![allow(clippy::boxed_local)]

mod benchmarking;
mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    traits::{IsSubType, UnfilteredDispatchable},
};
use scale_info::prelude::boxed::Box;
use sp_runtime::traits::Dispatchable;

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The overarching event type.
        type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The origin type required for performing upgradable operations.
        type UpgradableOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Type representing the weight of this pallet.
        type WeightInfo: WeightInfo;

        /// Type representing the worst case origin type used in weight benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type WorstCaseOriginType: Into<Self::RuntimeOrigin>;

        /// Retrieves the worst case origin for use in weight benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type WorstCaseOrigin: Get<Self::WorstCaseOriginType>;
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
        #[pallet::call_index(0)]
        #[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(
				T::WeightInfo::dispatch_as_root()
					.saturating_add(dispatch_info.call_weight).saturating_add(dispatch_info.extension_weight),
				dispatch_info.class,
			)
		})]
        pub fn dispatch_as_root(
            origin: OriginFor<T>,
            call: Box<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            T::UpgradableOrigin::ensure_origin(origin)?;

            let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());

            Self::deposit_event(Event::DispatchedAsRoot {
                result: res.map(|_| ()).map_err(|e| e.error),
            });
            Ok(Pays::No.into())
        }

        /// Dispatches a function call from root origin.
        /// This function does not check the weight of the call, and instead allows the
        /// caller to specify the weight of the call.
        ///
        /// The weight of this call is defined by the caller.
        #[pallet::call_index(1)]
        #[pallet::weight((*weight, call.get_dispatch_info().class))]
        pub fn dispatch_as_root_unchecked_weight(
            origin: OriginFor<T>,
            call: Box<<T as Config>::Call>,
            weight: Weight,
        ) -> DispatchResultWithPostInfo {
            let _ = weight; // We dont need to check the weight witness.
            T::UpgradableOrigin::ensure_origin(origin)?;

            let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());

            Self::deposit_event(Event::DispatchedAsRoot {
                result: res.map(|_| ()).map_err(|e| e.error),
            });
            Ok(Pays::No.into())
        }
    }
}
