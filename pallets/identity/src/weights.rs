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
    fn create_identity() -> Weight;
    fn confirm_identity() -> Weight;
    fn validate_identity() -> Weight;
    fn change_owner_key() -> Weight;
    fn revoke_identity() -> Weight;
    fn remove_identity() -> Weight;
    fn prune_item_identities_names(i: u32) -> Weight;
    fn fix_sufficients() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: Identity IdentityIndexOf (r:2 w:1)
    // Storage: Identity Identities (r:2 w:2)
    // Storage: Cert StorageIdtyCertMeta (r:2 w:2)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Identity NextIdtyIndex (r:1 w:1)
    // Storage: Identity CounterForIdentities (r:1 w:1)
    // Storage: Identity IdentitiesRemovableOn (r:1 w:1)
    // Storage: Cert StorageCertsRemovableOn (r:1 w:1)
    // Storage: Cert CertsByReceiver (r:1 w:1)
    fn create_identity() -> Weight {
        // Minimum execution time: 440_987 nanoseconds.
        Weight::from_ref_time(462_747_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(13 as u64))
            .saturating_add(RocksDbWeight::get().writes(11 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: Identity Identities (r:1 w:1)
    // Storage: Identity IdentitiesNames (r:1 w:1)
    // Storage: Membership PendingMembership (r:1 w:1)
    // Storage: Membership Membership (r:1 w:0)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership PendingMembershipsExpireOn (r:1 w:1)
    fn confirm_identity() -> Weight {
        // Minimum execution time: 186_617 nanoseconds.
        Weight::from_ref_time(309_527_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(7 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }
    // Storage: Identity Identities (r:1 w:1)
    // Storage: Membership Membership (r:1 w:1)
    // Storage: Cert StorageIdtyCertMeta (r:1 w:0)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership PendingMembership (r:1 w:1)
    // Storage: Membership CounterForMembership (r:1 w:1)
    // Storage: Membership MembershipsExpireOn (r:1 w:1)
    // Storage: UniversalDividend CurrentUdIndex (r:1 w:0)
    fn validate_identity() -> Weight {
        // Minimum execution time: 299_920 nanoseconds.
        Weight::from_ref_time(320_025_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(8 as u64))
            .saturating_add(RocksDbWeight::get().writes(5 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:2 w:2)
    // Storage: Identity Identities (r:1 w:1)
    // Storage: SmithMembership Membership (r:1 w:0)
    // Storage: System BlockHash (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: AuthorityMembers Members (r:1 w:0)
    fn change_owner_key() -> Weight {
        // Minimum execution time: 442_260 nanoseconds.
        Weight::from_ref_time(728_714_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(7 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }
    // Storage: Identity Identities (r:1 w:1)
    // Storage: SmithMembership Membership (r:1 w:0)
    // Storage: System BlockHash (r:1 w:0)
    // Storage: Membership Membership (r:1 w:1)
    // Storage: Identity CounterForIdentities (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Cert CertsByReceiver (r:1 w:1)
    // Storage: Cert StorageIdtyCertMeta (r:2 w:2)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Identity IdentityIndexOf (r:0 w:1)
    fn revoke_identity() -> Weight {
        // Minimum execution time: 494_407 nanoseconds.
        Weight::from_ref_time(800_824_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(10 as u64))
            .saturating_add(RocksDbWeight::get().writes(8 as u64))
    }
    // Storage: Identity Identities (r:1 w:1)
    // Storage: SmithMembership Membership (r:1 w:0)
    // Storage: Membership Membership (r:1 w:1)
    // Storage: Identity CounterForIdentities (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Cert CertsByReceiver (r:1 w:1)
    // Storage: Cert StorageIdtyCertMeta (r:2 w:2)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Identity IdentityIndexOf (r:0 w:1)
    // Storage: Identity IdentitiesNames (r:0 w:1)
    fn remove_identity() -> Weight {
        // Minimum execution time: 302_574 nanoseconds.
        Weight::from_ref_time(504_132_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(9 as u64))
            .saturating_add(RocksDbWeight::get().writes(9 as u64))
    }
    // Storage: Identity IdentitiesNames (r:0 w:20)
    /// The range of component `i` is `[1, 1000]`.
    fn prune_item_identities_names(i: u32) -> Weight {
        // Minimum execution time: 22_533 nanoseconds.
        Weight::from_ref_time(282_674_421 as u64)
            // Standard Error: 170_391
            .saturating_add(Weight::from_ref_time(5_660_460 as u64).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    // Storage: System Account (r:1 w:1)
    fn fix_sufficients() -> Weight {
        // Minimum execution time: 112_793 nanoseconds.
        Weight::from_ref_time(122_192_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
}