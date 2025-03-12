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

//! # Duniter v2s Documentation
//!
//! 🆙 A rewriting of [Duniter v1](https://duniter.org) in the [Substrate](https://www.substrate.io/) framework.
//!
//! ⚠️ Duniter-v2s is under active development.
//!
//! 🚧 A test network called "ĞDev" is deployed, allowing testing of wallets and indexers.
//!
//! ## Crate Overview
//!
//! This workspace consists of multiple crates that collaboratively implement the Duniter node, enabling features such as identity management, Web of Trust (WoT) evaluation, universal dividend calculation, and more. Below is a categorized list of crates within this workspace:
//!
//! ### Core Components
//! - [`client/distance`](../dc_distance/index.html): Provides an inherent data provider for distance evaluation in the Web of Trust.
//! - [`distance-oracle`](../distance_oracle/index.html): A standalone tool for performing computationally intensive Web of Trust distance calculations.
//! - [`node`](../duniter/index.html): The main node implementation for running the Duniter blockchain network.
//!
//! ### Testing Utilities
//! - [`end2end-tests`](../duniter_end2end_tests/index.html): End-to-end tests for validating the entire Duniter workflow.
//! - [`live-tests`](../duniter_live_tests/index.html): Live test cases for ensuring the integrity of the chain.
//!
//! ### Pallets (Runtime Modules)
//! - [`pallets/authority-members`](../pallet_authority_members/index.html): Manages the authority members.
//! - [`pallets/certification`](../pallet_certification/index.html): Handles identity certification.
//! - [`pallets/distance`](../pallet_distance/index.html): Implements the storage and logic for WoT distance calculations.
//! - [`pallets/duniter-test-parameters`](../pallet_duniter_test_parameters/index.html): Provides runtime testing parameters.
//! - [`pallets/duniter-test-parameters/macro`](../pallet_duniter_test_parameters_macro/index.html): Macros to simplify testing configurations.
//! - [`pallets/duniter-wot`](../pallet_duniter_wot/index.html): Core logic for managing the WoT.
//! - [`pallets/identity`](../pallet_identity/index.html): Implements identity management.
//! - [`pallets/membership`](../pallet_membership/index.html): Manages memberships.
//! - [`pallets/oneshot-account`](../pallet_oneshot_account/index.html): Manages one-shot accounts.
//! - [`pallets/quota`](../pallet_quota/index.html): Manages users quotas.
//! - [`pallets/smith-members`](../pallet_smith_members/index.html): Manages smiths.
//! - [`pallets/universal-dividend`](../pallet_universal_dividend/index.html): Handles the logic for distributing universal dividends.
//! - [`pallets/upgrade-origin`](../pallet_upgrade_origin/index.html): Ensures secure origins for runtime upgrades.
//!
//! ### Shared Primitives
//! - [`primitives/distance`](../sp_distance/index.html): Shared types and logic for distance evaluations.
//! - [`primitives/membership`](../sp_membership/index.html): Shared primitives for membership-related operations.
//!
//! ### Tooling and Utilities
//! - [`resources/weight_analyzer`](../weightanalyzer/index.html): Provides tools for analyzing runtime weights.
//! - [`runtime/common`](../common_runtime/index.html): Shared components and utilities used across multiple runtimes.
//! - [`runtime/gdev`](../gdev_runtime/index.html): The runtime implementation for the GDEV test network.
//! - [`runtime/g1`](../g1_runtime/index.html): The runtime implementation for the G1 test network.
//! - [`runtime/gtest`](../gtest_runtime/index.html): The runtime implementation for the GTEST test network.
//! - [`xtask`](../xtask/index.html): A custom xtask runner to automate release and testing.

pub mod chain_spec;
pub mod cli;
pub mod command;
pub mod endpoint_gossip;
pub mod rpc;
pub mod service;
