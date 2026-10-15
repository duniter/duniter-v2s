// Copyright 2021 Axiom-Team
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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[path = "../../shared/parameters.rs"]
pub mod parameters;
#[path = "../../shared/weights.rs"]
pub mod weights;

include!("../../shared/runtime.rs");

#[cfg(test)]
#[path = "../../common/src/proxy_tests.rs"]
mod proxy_tests;

define_runtime!("gtest", [Sudo: pallet_sudo = 20,], [[pallet_sudo, Sudo]]);

impl pallet_sudo::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = sudo_weights::WeightInfo<Runtime>;
}

#[path = "../../shared/weights/pallet_sudo.rs"]
mod sudo_weights;
