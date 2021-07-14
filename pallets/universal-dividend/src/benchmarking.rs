// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

//! Benchmarking setup for pallet-universal-dividend

use super::*;

#[allow(unused)]
use crate::Pallet as UniversalDividend;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

// Create state for use in `on_initialize`.
fn create_state<T: Config>(n: u32) -> Result<(), &'static str> {
    <LastReevalStorage<T>>::put(LastReeval {
        members_count: T::MembersCount::get(),
        monetary_mass: T::Currency::total_issuance(),
        ud_amount: new_ud_amount,
    });
    Ok(())
}

benchmarks! {
    create_ud {
        run_to_block(2);
    }: UniversalDividend::on_initialize()
    verify {
        assert_eq!(System::events().len(), 7);
    }
}

impl_benchmark_test_suite!(
    UniversalDividend,
    crate::mock::new_test_ext(UniversalDividendConfig {
        first_ud: 1_000,
        initial_monetary_mass: 0,
    }),
    crate::mock::Test,
);
