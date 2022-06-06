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

pub use frame_support::weights::{
    Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use sp_arithmetic::traits::{BaseArithmetic, One, Unsigned};

pub struct WeightToFeeImpl<T>(sp_std::marker::PhantomData<T>);

impl<T> WeightToFeePolynomial for WeightToFeeImpl<T>
where
    T: BaseArithmetic + From<u32> + Copy + Unsigned,
{
    type Balance = T;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec::smallvec!(WeightToFeeCoefficient {
            coeff_integer: 0u32.into(),
            coeff_frac: sp_runtime::Perbill::from_parts(1),
            negative: false,
            degree: 1,
        })
    }
    // Force constant fees
    fn calc(_weight: &Weight) -> Self::Balance {
        One::one()
    }
}
