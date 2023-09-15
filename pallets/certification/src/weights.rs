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
    fn add_cert() -> Weight;
    fn del_cert() -> Weight;
    fn remove_all_certs_received_by(i: u32) -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: Identity Identities (r:2 w:0)
    // Storage: Cert StorageIdtyCertMeta (r:2 w:2)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Cert StorageCertsRemovableOn (r:1 w:1)
    // Storage: Cert CertsByReceiver (r:1 w:1)
    fn add_cert() -> Weight {
        // Minimum execution time: 259_247 nanoseconds.
        Weight::from_parts(269_348_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(7 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }
    // Storage: Cert CertsByReceiver (r:1 w:1)
    // Storage: Cert StorageIdtyCertMeta (r:2 w:2)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership Membership (r:1 w:0)
    fn del_cert() -> Weight {
        // Minimum execution time: 216_762 nanoseconds.
        Weight::from_parts(222_570_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
    // Storage: Cert CertsByReceiver (r:1 w:1)
    // Storage: Cert StorageIdtyCertMeta (r:2 w:2)
    // Storage: Parameters ParametersStorage (r:1 w:0)
    // Storage: Membership Membership (r:1 w:0)
    /// The range of component `i` is `[2, 1000]`.
    fn remove_all_certs_received_by(i: u32) -> Weight {
        // Minimum execution time: 223_292 nanoseconds.
        Weight::from_parts(233_586_000 as u64, 0)
            // Standard Error: 598_929
            .saturating_add(Weight::from_parts(53_659_501 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(3 as u64))
            .saturating_add(RocksDbWeight::get().reads((1 as u64).saturating_mul(i as u64)))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
}
