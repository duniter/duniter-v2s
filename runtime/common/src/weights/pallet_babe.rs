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

//! Manual weights for the GRANDPA Pallet in duniter runtimes
//! This file was not auto-generated.

use frame_support::{
    traits::Get,
    weights::{
        constants::{WEIGHT_PER_MICROS, WEIGHT_PER_NANOS},
        Weight,
    },
};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_babe`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_babe::WeightInfo for WeightInfo<T> {
    fn plan_config_change() -> Weight {
        T::DbWeight::get().writes(1)
    }

    fn report_equivocation(validator_count: u32) -> Weight {
        // we take the validator set count from the membership proof to
        // calculate the weight but we set a floor of 100 validators.
        let validator_count = validator_count.max(100) as u64;

        // checking membership proof
        (35 * WEIGHT_PER_MICROS)
            .saturating_add((175 * WEIGHT_PER_NANOS).saturating_mul(validator_count))
            .saturating_add(T::DbWeight::get().reads(5))
            // check equivocation proof
            .saturating_add(110 * WEIGHT_PER_MICROS)
            // report offence
            .saturating_add(110 * WEIGHT_PER_MICROS)
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
