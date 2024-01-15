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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

pub mod block_weights;
pub mod extrinsic_weights;
pub mod frame_system;
pub mod pallet_babe;
pub mod pallet_balances;
pub mod pallet_collective;
pub mod pallet_distance;
pub mod pallet_grandpa;
pub mod pallet_im_online;
pub mod pallet_multisig;
pub mod pallet_proxy;
pub mod pallet_session;
pub mod pallet_scheduler;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_universal_dividend;
pub mod pallet_upgrade_origin;
pub mod pallet_provide_randomness;
pub mod pallet_identity;
pub mod pallet_preimage;
pub mod pallet_utility;
pub mod pallet_duniter_account;
pub mod pallet_quota;
pub mod pallet_oneshot_account;
pub mod pallet_certification_cert;
pub mod pallet_membership_membership;
pub mod pallet_smith_members;
pub mod pallet_authority_members;
pub mod paritydb_weights;
