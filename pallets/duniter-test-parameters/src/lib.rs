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

//! # Duniter Test Parameters Pallet
//!
//! This pallet allows ĞDev runtime to tweak parameter values instead of having it as runtime constants.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use types::*;

pub mod types {
    use super::{Config, Pallet};
    use codec::{Decode, Encode};
    use frame_support::pallet_prelude::*;
    use pallet_duniter_test_parameters_macro::generate_fields_getters;

    #[generate_fields_getters]
    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        scale_info::TypeInfo,
    )]
    pub struct Parameters<
        BlockNumber: Default + Parameter,
        CertCount: Default + Parameter,
        PeriodCount: Default + Parameter,
        SessionCount: Default + Parameter,
    > {
        pub babe_epoch_duration: PeriodCount,
        pub cert_period: BlockNumber,
        pub cert_max_by_issuer: CertCount,
        pub cert_min_received_cert_to_issue_cert: CertCount,
        pub cert_validity_period: BlockNumber,
        pub distance_evaluation_period: BlockNumber,
        pub idty_confirm_period: BlockNumber,
        pub idty_creation_period: BlockNumber,
        pub membership_period: BlockNumber,
        pub membership_renewal_period: BlockNumber,
        pub ud_creation_period: PeriodCount,
        pub ud_reeval_period: PeriodCount,
        pub smith_cert_max_by_issuer: CertCount,
        pub smith_wot_min_cert_for_membership: CertCount,
        pub smith_inactivity_max_duration: SessionCount,
        pub wot_first_cert_issuable_on: BlockNumber,
        pub wot_min_cert_for_create_idty_right: CertCount,
        pub wot_min_cert_for_membership: CertCount,
    }
}

#[allow(unreachable_patterns)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::StorageVersion};

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // CONFIG //

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type BlockNumber: Default + MaybeSerializeDeserialize + Parameter;
        type CertCount: Default + MaybeSerializeDeserialize + Parameter;
        type PeriodCount: Default + MaybeSerializeDeserialize + Parameter;
        type SessionCount: Default + MaybeSerializeDeserialize + Parameter;
    }

    // STORAGE //

    #[pallet::storage]
    #[pallet::getter(fn parameters)]
    pub type ParametersStorage<T: Config> = StorageValue<
        _,
        Parameters<T::BlockNumber, T::CertCount, T::PeriodCount, T::SessionCount>,
        ValueQuery,
    >;

    // GENESIS

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub parameters: Parameters<T::BlockNumber, T::CertCount, T::PeriodCount, T::SessionCount>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                parameters: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            <ParametersStorage<T>>::put(self.parameters.clone());
        }
    }
}
