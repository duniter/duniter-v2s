// Copyright 2021-2023 Axiom-Team
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
    fn force_request_membership() -> Weight;
    fn request_membership() -> Weight;
    fn claim_membership() -> Weight;
    fn renew_membership() -> Weight;
    fn revoke_membership() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: Membership PendingMembership (r:1 w:1)
    // Storage: Membership Membership (r:1 w:0)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership PendingMembershipsExpireOn (r:1 w:1)
    fn force_request_membership() -> Weight {
        // Minimum execution time: 89_725 nanoseconds.
        Weight::from_ref_time(98_333_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: Identity Identities (r:1 w:0)
    fn request_membership() -> Weight {
        // Minimum execution time: 48_477 nanoseconds.
        Weight::from_ref_time(50_689_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: Membership Membership (r:1 w:1)
    // Storage: Cert StorageIdtyCertMeta (r:1 w:0)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership PendingMembership (r:1 w:1)
    // Storage: Membership CounterForMembership (r:1 w:1)
    // Storage: Membership MembershipsExpireOn (r:1 w:1)
    fn claim_membership() -> Weight {
        // Minimum execution time: 144_079 nanoseconds.
        Weight::from_ref_time(146_565_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(7 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: Membership Membership (r:1 w:1)
    // Storage: Identity Identities (r:1 w:0)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership MembershipsExpireOn (r:1 w:1)
    fn renew_membership() -> Weight {
        // Minimum execution time: 120_859 nanoseconds.
        Weight::from_ref_time(124_222_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: Membership Membership (r:1 w:1)
    // Storage: Membership CounterForMembership (r:1 w:1)
    // Storage: Identity Identities (r:1 w:0)
    // Storage: UniversalDividend CurrentUdIndex (r:1 w:0)
    fn revoke_membership() -> Weight {
        // Minimum execution time: 109_486 nanoseconds.
        Weight::from_ref_time(113_303_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
}
