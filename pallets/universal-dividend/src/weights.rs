// Copyright 2021-2022 Axiom-Team
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

#![allow(clippy::unnecessary_cast)]

use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for pallet_universal_dividend.
pub trait WeightInfo {
    fn on_initialize() -> Weight;
    fn on_initialize_ud_created() -> Weight;
    fn on_initialize_ud_reevalued() -> Weight;
    fn claim_uds(n: u32) -> Weight;
    fn transfer_ud() -> Weight;
    fn transfer_ud_keep_alive() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: (r:0 w:0)
    fn on_initialize() -> Weight {
        Weight::from_ref_time(2_260_000)
    }
    // Storage: Membership CounterForMembership (r:1 w:0)
    // Storage: UniversalDividend NextReeval (r:1 w:0)
    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: UniversalDividend MonetaryMass (r:1 w:1)
    // Storage: UniversalDividend CurrentUdIndex (r:1 w:1)
    fn on_initialize_ud_created() -> Weight {
        Weight::from_ref_time(20_160_000)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    // Storage: Membership CounterForMembership (r:1 w:0)
    // Storage: UniversalDividend NextReeval (r:1 w:1)
    // Storage: UniversalDividend CurrentUd (r:1 w:1)
    // Storage: UniversalDividend MonetaryMass (r:1 w:1)
    // Storage: UniversalDividend PastReevals (r:1 w:1)
    // Storage: UniversalDividend CurrentUdIndex (r:1 w:1)
    fn on_initialize_ud_reevalued() -> Weight {
        Weight::from_ref_time(32_770_000)
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(5))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: Identity Identities (r:1 w:1)
    // Storage: UniversalDividend CurrentUdIndex (r:1 w:0)
    // Storage: UniversalDividend PastReevals (r:1 w:0)
    fn claim_uds(n: u32) -> Weight {
        Weight::from_ref_time(32_514_000)
            // Standard Error: 32_000
            .saturating_add(Weight::from_ref_time(8_000).saturating_mul(n as u64))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Account PendingNewAccounts (r:0 w:1)
    fn transfer_ud() -> Weight {
        Weight::from_ref_time(53_401_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Account PendingNewAccounts (r:0 w:1)
    fn transfer_ud_keep_alive() -> Weight {
        Weight::from_ref_time(33_420_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}
