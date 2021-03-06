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

pub use frame_support::weights::{Weight, WeightToFee};
use sp_arithmetic::traits::{BaseArithmetic, One, Unsigned};

pub struct LengthToFeeImpl<T>(sp_std::marker::PhantomData<T>);

impl<T> WeightToFee for LengthToFeeImpl<T>
where
    T: BaseArithmetic + From<u32> + Copy + Unsigned,
{
    type Balance = T;

    // Force constant fees
    fn weight_to_fee(lenght_in_bytes: &Weight) -> Self::Balance {
        (*lenght_in_bytes as u32 / 1_000_u32).into()
    }
}

pub struct WeightToFeeImpl<T>(sp_std::marker::PhantomData<T>);

impl<T> WeightToFee for WeightToFeeImpl<T>
where
    T: BaseArithmetic + From<u32> + Copy + Unsigned,
{
    type Balance = T;

    fn weight_to_fee(_weight: &Weight) -> Self::Balance {
        // Force constant fees for now
        One::one()
    }
}
