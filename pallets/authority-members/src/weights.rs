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
    fn go_offline() -> Weight;
    fn go_online() -> Weight;
    fn set_session_keys() -> Weight;
    fn remove_member() -> Weight;
    fn remove_member_from_blacklist() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: SmithMembership Membership (r:1 w:0)
    // Storage: AuthorityMembers Members (r:1 w:0)
    // Storage: AuthorityMembers OutgoingAuthorities (r:1 w:1)
    // Storage: AuthorityMembers IncomingAuthorities (r:1 w:0)
    // Storage: AuthorityMembers OnlineAuthorities (r:1 w:0)
    // Storage: AuthorityMembers AuthoritiesCounter (r:1 w:1)
    fn go_offline() -> Weight {
        // Minimum execution time: 120_876 nanoseconds.
        Weight::from_parts(122_190_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(7 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: SmithMembership Membership (r:1 w:0)
    // Storage: AuthorityMembers Members (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:0)
    // Storage: AuthorityMembers IncomingAuthorities (r:1 w:1)
    // Storage: AuthorityMembers OutgoingAuthorities (r:1 w:0)
    // Storage: AuthorityMembers OnlineAuthorities (r:1 w:0)
    // Storage: AuthorityMembers AuthoritiesCounter (r:1 w:1)
    fn go_online() -> Weight {
        // Minimum execution time: 145_521 nanoseconds.
        Weight::from_parts(157_428_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(8 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    // Storage: Identity IdentityIndexOf (r:1 w:0)
    // Storage: SmithMembership Membership (r:1 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:1)
    // Storage: Session KeyOwner (r:4 w:0)
    // Storage: Session CurrentIndex (r:1 w:0)
    // Storage: AuthorityMembers Members (r:1 w:1)
    // Storage: AuthorityMembers MustRotateKeysBefore (r:1 w:1)
    fn set_session_keys() -> Weight {
        // Minimum execution time: 181_682 nanoseconds.
        Weight::from_parts(192_995_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(11 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
    // Storage: AuthorityMembers Members (r:1 w:1)
    // Storage: AuthorityMembers OnlineAuthorities (r:1 w:1)
    // Storage: AuthorityMembers OutgoingAuthorities (r:1 w:1)
    // Storage: AuthorityMembers AuthoritiesCounter (r:1 w:1)
    // Storage: AuthorityMembers IncomingAuthorities (r:1 w:1)
    // Storage: Session NextKeys (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: SmithMembership Membership (r:1 w:1)
    // Storage: SmithMembership CounterForMembership (r:1 w:1)
    // Storage: Session KeyOwner (r:0 w:4)
    fn remove_member() -> Weight {
        // Minimum execution time: 246_592 nanoseconds.
        Weight::from_parts(256_761_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(9 as u64))
            .saturating_add(RocksDbWeight::get().writes(13 as u64))
    }
    // Storage: AuthorityMembers BlackList (r:1 w:1)
    fn remove_member_from_blacklist() -> Weight {
        // Minimum execution time: 60_023 nanoseconds.
        Weight::from_parts(60_615_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
}
