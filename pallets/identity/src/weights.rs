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
    fn force_remove_identity() -> Weight;
    fn prune_item_identities_names(i: u32) -> Weight;
    fn fix_sufficients() -> Weight;
    fn link_account() -> Weight;
    fn on_initialize() -> Weight;
    fn do_remove_identity_noop() -> Weight;
    fn do_remove_identity() -> Weight;
    fn prune_identities_noop() -> Weight;
    fn prune_identities_none() -> Weight;
    fn prune_identities_err() -> Weight;
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
        Weight::from_parts(462_747_000 as u64, 0)
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
        Weight::from_parts(309_527_000 as u64, 0)
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
        Weight::from_parts(320_025_000 as u64, 0)
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
        Weight::from_parts(728_714_000 as u64, 0)
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
        Weight::from_parts(800_824_000 as u64, 0)
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
    fn force_remove_identity() -> Weight {
        // Minimum execution time: 302_574 nanoseconds.
        Weight::from_parts(504_132_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(9 as u64))
            .saturating_add(RocksDbWeight::get().writes(9 as u64))
    }
    // Storage: Identity IdentitiesNames (r:0 w:20)
    /// The range of component `i` is `[1, 1000]`.
    fn prune_item_identities_names(i: u32) -> Weight {
        // Minimum execution time: 22_533 nanoseconds.
        Weight::from_parts(282_674_421 as u64, 0)
            // Standard Error: 170_391
            .saturating_add(Weight::from_parts(5_660_460 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    // Storage: System Account (r:1 w:1)
    fn fix_sufficients() -> Weight {
        // Minimum execution time: 112_793 nanoseconds.
        Weight::from_parts(122_192_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    /// Storage: Identity IdentityIndexOf (r:1 w:0)
    /// Proof Skipped: Identity IdentityIndexOf (max_values: None, max_size: None, mode: Measured)
    /// Storage: System BlockHash (r:1 w:0)
    /// Proof: System BlockHash (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(126), added: 2601, mode: MaxEncodedLen)
    fn link_account() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn on_initialize() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn do_remove_identity_noop() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn do_remove_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn prune_identities_noop() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn prune_identities_none() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn prune_identities_err() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `359`
        //  Estimated: `3824`
        // Minimum execution time: 543_046_000 picoseconds.
        Weight::from_parts(544_513_000, 0)
            .saturating_add(Weight::from_parts(0, 3824))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
