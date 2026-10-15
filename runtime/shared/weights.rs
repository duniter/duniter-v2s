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
#![allow(unused_doc_comments)]

#[path = "weights/frame_system_extensions.rs"]
pub mod frame_system_extensions;
#[path = "weights/pallet_transaction_payment.rs"]
pub mod pallet_transaction_payment;
#[path = "weights/block_weights.rs"]
pub mod block_weights;
#[path = "weights/extrinsic_weights.rs"]
pub mod extrinsic_weights;
#[path = "weights/frame_system.rs"]
pub mod frame_system;
#[path = "weights/pallet_babe.rs"]
pub mod pallet_babe;
#[path = "weights/pallet_balances.rs"]
pub mod pallet_balances;
#[path = "weights/pallet_collective.rs"]
pub mod pallet_collective;
#[path = "weights/pallet_distance.rs"]
pub mod pallet_distance;
#[path = "weights/pallet_grandpa.rs"]
pub mod pallet_grandpa;
#[path = "weights/pallet_im_online.rs"]
pub mod pallet_im_online;
#[path = "weights/pallet_multisig.rs"]
pub mod pallet_multisig;
#[path = "weights/pallet_proxy.rs"]
pub mod pallet_proxy;
#[path = "weights/pallet_session.rs"]
pub mod pallet_session;
#[path = "weights/pallet_scheduler.rs"]
pub mod pallet_scheduler;
#[path = "weights/pallet_timestamp.rs"]
pub mod pallet_timestamp;
#[path = "weights/pallet_treasury.rs"]
pub mod pallet_treasury;
#[path = "weights/pallet_universal_dividend.rs"]
pub mod pallet_universal_dividend;
#[path = "weights/pallet_upgrade_origin.rs"]
pub mod pallet_upgrade_origin;
#[path = "weights/pallet_provide_randomness.rs"]
pub mod pallet_provide_randomness;
#[path = "weights/pallet_identity.rs"]
pub mod pallet_identity;
#[path = "weights/pallet_preimage.rs"]
pub mod pallet_preimage;
#[path = "weights/pallet_utility.rs"]
pub mod pallet_utility;
#[path = "weights/pallet_duniter_account.rs"]
pub mod pallet_duniter_account;
#[path = "weights/pallet_quota.rs"]
pub mod pallet_quota;
#[path = "weights/pallet_oneshot_account.rs"]
pub mod pallet_oneshot_account;
#[path = "weights/pallet_certification.rs"]
pub mod pallet_certification;
#[path = "weights/pallet_membership.rs"]
pub mod pallet_membership;
#[path = "weights/pallet_smith_members.rs"]
pub mod pallet_smith_members;

#[path = "weights/pallet_authority_members.rs"]
pub mod pallet_authority_members;
#[path = "weights/paritydb_weights.rs"]
pub mod paritydb_weights;
