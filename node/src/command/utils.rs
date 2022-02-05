// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use sc_cli::{Error, SubstrateCli};
use sp_io::hashing::twox_128;

#[derive(Debug, clap::Subcommand)]
pub enum UtilsSubCommand {
    /// Compute the raw storage key prefix
    StorageKeyPrefix(StorageKeyPrefixCmd),
}

impl UtilsSubCommand {
    /// Run the command
    pub fn run<C: SubstrateCli>(&self, cli: &C) -> Result<(), Error> {
        match self {
            Self::StorageKeyPrefix(cmd) => cmd.run(cli),
        }
    }
}

#[derive(Debug, clap::Parser)]
pub struct StorageKeyPrefixCmd {
    /// Pallet name
    #[clap(short = 'p', long)]
    pallet_name: Option<String>,
    /// Storage item name
    #[clap(short = 'i', long)]
    item_name: Option<String>,
}

impl StorageKeyPrefixCmd {
    /// Run the command
    pub fn run<C: SubstrateCli>(&self, _cli: &C) -> Result<(), Error> {
        let mut key_prefix = Vec::new();
        let mut print_key_prefix = false;
        if let Some(ref pallet_name) = self.pallet_name {
            print_key_prefix = true;
            let pallet_prefix = twox_128(pallet_name.as_bytes());
            println!("Pallet prefix: 0x{}", hex::encode(&pallet_prefix));
            key_prefix.extend_from_slice(&pallet_prefix[..]);
        }
        if let Some(ref item_name) = self.item_name {
            let item_prefix = twox_128(item_name.as_bytes());
            println!("Item prefix: 0x{}", hex::encode(item_prefix));
            key_prefix.extend_from_slice(&item_prefix[..]);
        }
        if print_key_prefix {
            println!("Key prefix: 0x{}", hex::encode(key_prefix));
        }

        Ok(())
    }
}
