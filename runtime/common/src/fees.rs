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
use sp_arithmetic::traits::{BaseArithmetic, Unsigned};

#[cfg(not(feature = "constant-fees"))]
use {
    crate::weights::extrinsic_weights::ExtrinsicBaseWeight,
    frame_support::weights::{
        WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    smallvec::smallvec,
    sp_arithmetic::MultiplyRational,
    sp_runtime::Perbill,
    sp_runtime::SaturatedConversion,
};

pub struct LengthToFeeImpl<T>(sp_std::marker::PhantomData<T>);

impl<T> WeightToFee for LengthToFeeImpl<T>
where
    T: BaseArithmetic + From<u32> + Copy + Unsigned,
{
    type Balance = T;

    #[cfg(not(feature = "constant-fees"))]
    fn weight_to_fee(length_in_bytes: &Weight) -> Self::Balance {
        Self::Balance::saturated_from(length_in_bytes.ref_time() / 100u64)
    }

    #[cfg(feature = "constant-fees")]
    fn weight_to_fee(_length_in_bytes: &Weight) -> Self::Balance {
        0u32.into()
    }
}

pub struct WeightToFeeImpl<T>(sp_std::marker::PhantomData<T>);

#[cfg(not(feature = "constant-fees"))]
impl<T> WeightToFeePolynomial for WeightToFeeImpl<T>
where
    T: BaseArithmetic + From<u64> + Copy + Unsigned + From<u32> + MultiplyRational,
{
    type Balance = T;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // The extrinsic base weight (smallest non-zero weight) is mapped to 5 cent
        let p: Self::Balance = 5u64.into();
        let q: Self::Balance = Self::Balance::from(ExtrinsicBaseWeight::get().ref_time());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

#[cfg(feature = "constant-fees")]
impl<T> WeightToFee for WeightToFeeImpl<T>
where
    T: BaseArithmetic + From<u32> + Copy + Unsigned,
{
    type Balance = T;

    fn weight_to_fee(_weight: &Weight) -> Self::Balance {
        1u32.into()
    }
}
