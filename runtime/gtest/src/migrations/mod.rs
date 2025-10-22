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

//! # Runtime Migrations
//!
//! This module contains all migrations for the gtest runtime.
//! Each migration is versioned according to the spec_version it targets.
//!
//! ## Naming Convention
//!
//! - Module name: `v<spec_version>` (e.g., `v1110`, `v1120`)
//! - Struct name: Descriptive of what the migration does (e.g., `FixUdReevalDate`)
//!
//! ## Migration Order
//!
//! Migrations are executed in the order they are listed in the `Migrations` tuple.
//! Always add new migrations at the END of the tuple to maintain order.

pub mod v1110;

#[cfg(test)]
mod tests;
