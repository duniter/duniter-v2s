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

pub use pallet::*;
pub use types::*;

pub mod types {
    use super::{Config, Pallet};
    use codec::{Decode, Encode};
    use frame_support::pallet_prelude::*;
    use pallet_duniter_test_parameters_macro::generate_fields_getters;
    use scale_info::TypeInfo;
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    #[generate_fields_getters]
    #[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
    #[derive(Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct Parameters<
        BlockNumber: Default + Parameter,
        CertCount: Default + Parameter,
        PeriodCount: Default + Parameter,
    > {
        pub babe_epoch_duration: PeriodCount,
        pub cert_period: BlockNumber,
        pub cert_max_by_issuer: CertCount,
        pub cert_min_received_cert_to_issue_cert: CertCount,
        pub cert_renewable_period: BlockNumber,
        pub cert_validity_period: BlockNumber,
        pub idty_confirm_period: BlockNumber,
        pub idty_creation_period: BlockNumber,
        pub idty_max_disabled_period: BlockNumber,
        pub membership_period: BlockNumber,
        pub membership_renewable_period: BlockNumber,
        pub pending_membership_period: BlockNumber,
        pub ud_creation_period: BlockNumber,
        pub ud_first_reeval: BlockNumber,
        pub ud_reeval_period: PeriodCount,
        pub ud_reeval_period_in_blocks: BlockNumber,
        pub smith_cert_period: BlockNumber,
        pub smith_cert_max_by_issuer: CertCount,
        pub smith_cert_min_received_cert_to_issue_cert: CertCount,
        pub smith_cert_renewable_period: BlockNumber,
        pub smith_cert_validity_period: BlockNumber,
        pub smith_membership_period: BlockNumber,
        pub smith_membership_renewable_period: BlockNumber,
        pub smith_pending_membership_period: BlockNumber,
        pub smiths_wot_first_cert_issuable_on: BlockNumber,
        pub smiths_wot_min_cert_for_membership: CertCount,
        pub wot_first_cert_issuable_on: BlockNumber,
        pub wot_min_cert_for_create_idty_right: CertCount,
        pub wot_min_cert_for_membership: CertCount,
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::StorageVersion;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type CertCount: Default + MaybeSerializeDeserialize + Parameter;
        type PeriodCount: Default + MaybeSerializeDeserialize + Parameter;
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn parameters)]
    pub type ParametersStorage<T: Config> =
        StorageValue<_, Parameters<T::BlockNumber, T::CertCount, T::PeriodCount>, ValueQuery>;

    // GENESIS

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub parameters: Parameters<T::BlockNumber, T::CertCount, T::PeriodCount>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                parameters: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <ParametersStorage<T>>::put(self.parameters.clone());
        }
    }
}
