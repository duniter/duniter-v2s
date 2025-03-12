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

//! Duniter Node CLI.

#![warn(missing_docs)]

//mod benchmarking;
mod chain_spec;
#[macro_use]
mod service;
pub(crate) mod cli;
mod command;
mod endpoint_gossip;
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
