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

pub trait WeightInfo {
    fn create_identity() -> Weight;
    fn confirm_identity() -> Weight;
    fn change_owner_key() -> Weight;
    fn revoke_identity() -> Weight;
    fn prune_item_identities_names(i: u32) -> Weight;
    fn fix_sufficients() -> Weight;
    fn link_account() -> Weight;
    fn on_initialize() -> Weight;
    fn do_revoke_identity_noop() -> Weight;
    fn do_revoke_identity() -> Weight;
    fn do_remove_identity_noop() -> Weight;
    fn do_remove_identity() -> Weight;
    fn prune_identities_noop() -> Weight;
    fn prune_identities_none() -> Weight;
    fn prune_identities_err() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    fn create_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1165`
        //  Estimated: `7105`
        // Minimum execution time: 1_643_969_000 picoseconds.
        Weight::from_parts(1_781_521_000, 0)
            .saturating_add(Weight::from_parts(0, 7105))
            .saturating_add(RocksDbWeight::get().reads(14))
            .saturating_add(RocksDbWeight::get().writes(12))
    }

    fn confirm_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `661`
        //  Estimated: `6601`
        // Minimum execution time: 564_892_000 picoseconds.
        Weight::from_parts(588_761_000, 0)
            .saturating_add(Weight::from_parts(0, 6601))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    fn change_owner_key() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `837`
        //  Estimated: `6777`
        // Minimum execution time: 991_641_000 picoseconds.
        Weight::from_parts(1_071_332_000, 0)
            .saturating_add(Weight::from_parts(0, 6777))
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    fn revoke_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `778`
        //  Estimated: `6718`
        // Minimum execution time: 829_174_000 picoseconds.
        Weight::from_parts(869_308_000, 0)
            .saturating_add(Weight::from_parts(0, 6718))
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(6))
    }

    fn prune_item_identities_names(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 51_362_000 picoseconds.
        Weight::from_parts(80_389_000, 0)
            .saturating_add(Weight::from_parts(0, 0))
            // Standard Error: 75_232
            .saturating_add(Weight::from_parts(30_016_649, 0).saturating_mul(i.into()))
            .saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }

    fn fix_sufficients() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `67`
        //  Estimated: `3591`
        // Minimum execution time: 154_343_000 picoseconds.
        Weight::from_parts(156_117_000, 0)
            .saturating_add(Weight::from_parts(0, 3591))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn link_account() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `307`
        //  Estimated: `3772`
        // Minimum execution time: 538_773_000 picoseconds.
        Weight::from_parts(591_354_000, 0)
            .saturating_add(Weight::from_parts(0, 3772))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn on_initialize() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 4_529_000 picoseconds.
        Weight::from_parts(7_360_000, 0).saturating_add(Weight::from_parts(0, 0))
    }

    fn do_revoke_identity_noop() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `269`
        //  Estimated: `3734`
        // Minimum execution time: 103_668_000 picoseconds.
        Weight::from_parts(107_679_000, 0)
            .saturating_add(Weight::from_parts(0, 3734))
            .saturating_add(RocksDbWeight::get().reads(1))
    }

    fn do_revoke_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1525`
        //  Estimated: `7465`
        // Minimum execution time: 2_204_911_000 picoseconds.
        Weight::from_parts(2_225_493_000, 0)
            .saturating_add(Weight::from_parts(0, 7465))
            .saturating_add(RocksDbWeight::get().reads(17))
            .saturating_add(RocksDbWeight::get().writes(20))
    }

    fn do_remove_identity_noop() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `269`
        //  Estimated: `3734`
        // Minimum execution time: 104_296_000 picoseconds.
        Weight::from_parts(115_316_000, 0)
            .saturating_add(Weight::from_parts(0, 3734))
            .saturating_add(RocksDbWeight::get().reads(1))
    }

    fn do_remove_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1432`
        //  Estimated: `6192`
        // Minimum execution time: 2_870_497_000 picoseconds.
        Weight::from_parts(4_159_994_000, 0)
            .saturating_add(Weight::from_parts(0, 6192))
            .saturating_add(RocksDbWeight::get().reads(16))
            .saturating_add(RocksDbWeight::get().writes(22))
    }

    fn prune_identities_noop() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `108`
        //  Estimated: `3573`
        // Minimum execution time: 68_859_000 picoseconds.
        Weight::from_parts(71_836_000, 0)
            .saturating_add(Weight::from_parts(0, 3573))
            .saturating_add(RocksDbWeight::get().reads(1))
    }

    fn prune_identities_none() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `292`
        //  Estimated: `3757`
        // Minimum execution time: 178_332_000 picoseconds.
        Weight::from_parts(186_982_000, 0)
            .saturating_add(Weight::from_parts(0, 3757))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn prune_identities_err() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1177`
        //  Estimated: `4642`
        // Minimum execution time: 1_427_848_000 picoseconds.
        Weight::from_parts(2_637_229_000, 0)
            .saturating_add(Weight::from_parts(0, 4642))
            .saturating_add(RocksDbWeight::get().reads(8))
            .saturating_add(RocksDbWeight::get().writes(8))
    }
}
