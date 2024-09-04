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

// In the deployed fees model, a mapping of 5 (5cG) corresponds to a base extrinsic weight,
// achieved through a 1-dimensional polynomial. Additionally, 1 (1cG) corresponds to an extrinsic length of 100 bytes.
//
// For testing purposes, we adopt a human-predictable weight system that remains invariant to the chosen fees model for release.
// This involves setting a constant weight_to_fee equal to 1 and a constant length_to_fee set to 0, resulting in each extrinsic costing 2 (2cG).

pub use frame_support::weights::{Weight, WeightToFee};
use pallet_transaction_payment::{Multiplier, MultiplierUpdate};
use sp_arithmetic::traits::{BaseArithmetic, Unsigned};
use sp_core::Get;
use sp_runtime::{traits::Convert, Perquintill};
#[cfg(not(feature = "constant-fees"))]
use {
    frame_support::pallet_prelude::DispatchClass,
    frame_support::weights::{
        WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    smallvec::smallvec,
    sp_arithmetic::MultiplyRational,
    sp_runtime::traits::One,
    sp_runtime::Perbill,
    sp_runtime::SaturatedConversion,
    sp_runtime::Saturating,
};

/// A structure to implement Length to Fee conversion.
/// - `Balance`: The balance type.
/// - `Runtime`: The system configuration type, providing access to block weights.
/// - `Target`: A type providing the target block fullness.
pub struct LengthToFeeImpl<Balance, Runtime, Target>(
    sp_std::marker::PhantomData<Balance>,
    sp_std::marker::PhantomData<Runtime>,
    sp_std::marker::PhantomData<Target>,
);

/// Trait implementation for converting transaction length to fee.
impl<Balance, Runtime, Target> WeightToFee for LengthToFeeImpl<Balance, Runtime, Target>
where
    Balance: BaseArithmetic + From<u32> + Copy + Unsigned,
    Runtime: frame_system::Config + pallet_transaction_payment::Config,
    Target: Get<Perquintill>,
{
    type Balance = Balance;

    /// Function to convert weight to fee when "constant-fees" feature is not enabled.
    ///
    /// This function calculates the fee based on the length of the transaction in bytes.
    /// If the current block weight and length are less than a fraction of the max block weight and length and the fee multiplier is one,
    /// it returns a zero fee. Otherwise, it calculates the fee based on the length in bytes.
    #[cfg(not(feature = "constant-fees"))]
    fn weight_to_fee(length_in_bytes: &Weight) -> Self::Balance {
        let weights = Runtime::BlockWeights::get();
        let fee_multiplier = pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier();
        let normal_max_weight = weights
            .get(DispatchClass::Normal)
            .max_total
            .unwrap_or(weights.max_block);
        let current_block_weight = <frame_system::Pallet<Runtime>>::block_weight();

        let length = Runtime::BlockLength::get();
        let normal_max_length = *length.max.get(DispatchClass::Normal) as u64;
        let current_block_length = <frame_system::Pallet<Runtime>>::all_extrinsics_len() as u64;

        if current_block_weight
            .get(DispatchClass::Normal)
            .all_lt(Target::get() * normal_max_weight)
            && current_block_length < (Target::get() * normal_max_length)
            && fee_multiplier.is_one()
        {
            0u32.into()
        } else {
            Self::Balance::saturated_from(length_in_bytes.ref_time() / 100u64)
        }
    }

    /// Function to convert weight to fee when "constant-fees" feature is enabled.
    ///
    /// This function always returns a constant fee of zero when the "constant-fees" feature is enabled.
    #[cfg(feature = "constant-fees")]
    fn weight_to_fee(_length_in_bytes: &Weight) -> Self::Balance {
        0u32.into()
    }
}

/// A structure to implement Weight to Fee conversion.
/// - `Balance`: The balance type.
/// - `Runtime`: The system configuration type, providing access to block weights.
/// - `Target`: A type providing the target block fullness.
pub struct WeightToFeeImpl<Balance, Runtime, Target>(
    sp_std::marker::PhantomData<Balance>,
    sp_std::marker::PhantomData<Runtime>,
    sp_std::marker::PhantomData<Target>,
);

/// Trait implementation for converting transaction weight to fee.
///
/// This implementation is only included when the "constant-fees" feature is not enabled.
#[cfg(not(feature = "constant-fees"))]
impl<Balance, Runtime, Target> WeightToFeePolynomial for WeightToFeeImpl<Balance, Runtime, Target>
where
    Balance: BaseArithmetic + From<u64> + Copy + Unsigned + From<u32> + MultiplyRational,
    Runtime: frame_system::Config + pallet_transaction_payment::Config,
    Target: Get<Perquintill>,
{
    type Balance = Balance;

    /// Function to get the polynomial coefficients for weight to fee conversion.
    ///
    /// This function calculates the polynomial coefficients for converting transaction weight to fee.
    /// If the current block weight and length are less than a fraction of the block max weight and length, and the fee multiplier is one,
    /// it returns zero. Otherwise, it calculates the coefficients based on the extrinsic base weight mapped to 5 cents.
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let weights = Runtime::BlockWeights::get();
        let fee_multiplier = pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier();
        let normal_max_weight = weights
            .get(DispatchClass::Normal)
            .max_total
            .unwrap_or(weights.max_block);
        let current_block_weight = <frame_system::Pallet<Runtime>>::block_weight();

        let length = Runtime::BlockLength::get();
        let normal_max_length = *length.max.get(DispatchClass::Normal) as u64;
        let current_block_length = <frame_system::Pallet<Runtime>>::all_extrinsics_len() as u64;

        if current_block_weight
            .get(DispatchClass::Normal)
            .all_lt(Target::get() * normal_max_weight)
            && current_block_length < (Target::get() * normal_max_length)
            && fee_multiplier.is_one()
        {
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: Perbill::zero(),
                coeff_integer: Self::Balance::zero(),
            }]
        } else {
            // The extrinsic base weight (smallest non-zero weight) is mapped to 5 cents
            let p: Self::Balance = 5u64.into();
            let q: Self::Balance =
                Self::Balance::from(weights.get(DispatchClass::Normal).base_extrinsic.ref_time());
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: Perbill::from_rational(p % q, q),
                coeff_integer: p / q,
            }]
        }
    }
}

/// Trait implementation for converting transaction weight to a constant fee.
///
/// This implementation is only included when the "constant-fees" feature is enabled.
#[cfg(feature = "constant-fees")]
impl<Balance, Runtime, Target> WeightToFee for WeightToFeeImpl<Balance, Runtime, Target>
where
    Balance: BaseArithmetic + From<u32> + Copy + Unsigned,
{
    type Balance = Balance;

    fn weight_to_fee(_weight: &Weight) -> Self::Balance {
        1u32.into()
    }
}

/// A structure to implement fee multiplier adjustments.
///
/// - `Runtime`: The system configuration type.
/// - `Target`: A type providing the target block fullness.
/// - `MaxMultiplier`: A type providing the maximum multiplier value.
pub struct FeeMultiplier<Runtime, Target, MaxMultiplier>(
    sp_std::marker::PhantomData<Runtime>,
    sp_std::marker::PhantomData<Target>,
    sp_std::marker::PhantomData<MaxMultiplier>,
);

/// Trait implementation for updating the fee multiplier.
impl<Runtime, Target, MaxMultiplier> MultiplierUpdate
    for FeeMultiplier<Runtime, Target, MaxMultiplier>
where
    Runtime: frame_system::Config,
    Target: Get<Perquintill>,
    MaxMultiplier: Get<Multiplier>,
{
    fn min() -> Multiplier {
        0.into()
    }

    fn max() -> Multiplier {
        MaxMultiplier::get()
    }

    fn target() -> Perquintill {
        Target::get()
    }

    fn variability() -> Multiplier {
        Default::default()
    }
}

/// Trait implementation for converting previous `Multiplier` to another for fee adjustment.
impl<Runtime, Target, MaxMultiplier> Convert<Multiplier, Multiplier>
    for FeeMultiplier<Runtime, Target, MaxMultiplier>
where
    Runtime: frame_system::Config,
    Target: Get<Perquintill>,
    MaxMultiplier: Get<Multiplier>,
{
    /// Function to convert the previous fee multiplier to a new fee multiplier.
    ///
    /// This function adjusts the fee multiplier based on the current block weight, length and target block fullness.
    /// - If the current block weight and length are less than the target, it decreases the multiplier by one, with a minimum of one.
    /// - If the current block weight or length is more than the target, it increases the multiplier by one, up to the maximum multiplier.
    #[cfg(not(feature = "constant-fees"))]
    fn convert(previous: Multiplier) -> Multiplier {
        let max_multiplier = MaxMultiplier::get();
        let target_block_fullness = Target::get();

        let weights = Runtime::BlockWeights::get();
        let normal_max_weight = weights
            .get(DispatchClass::Normal)
            .max_total
            .unwrap_or(weights.max_block);

        let length = Runtime::BlockLength::get();
        let normal_max_length = *length.max.get(DispatchClass::Normal) as u64;
        let current_block_length = <frame_system::Pallet<Runtime>>::all_extrinsics_len() as u64;

        if <frame_system::Pallet<Runtime>>::block_weight()
            .get(DispatchClass::Normal)
            .all_lt(target_block_fullness * normal_max_weight)
            && current_block_length < (target_block_fullness * normal_max_length)
        {
            // If the current block weight and length are less than the target, keep the
            // multiplier at the minimum or decrease it by one to slowly
            // return to the minimum.
            previous.saturating_sub(1.into()).max(1.into())
        } else {
            // If the current block weight or length is more than the target, increase
            // the multiplier by one.
            previous.saturating_add(1.into()).min(max_multiplier)
        }
    }

    /// Function to convert the previous fee multiplier to a constant fee multiplier when "constant-fees" feature is enabled.
    ///
    /// This function always returns a constant multiplier of 1 when the "constant-fees" feature is enabled.
    #[cfg(feature = "constant-fees")]
    fn convert(_previous: Multiplier) -> Multiplier {
        1.into()
    }
}
