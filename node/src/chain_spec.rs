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

// Common to all Duniter blockchains
pub mod gen_genesis_data;

#[cfg(feature = "g1")]
pub mod g1;
#[cfg(feature = "gdev")]
pub mod gdev;
#[cfg(feature = "gtest")]
pub mod gtest;

use common_runtime::{AccountId, Signature};
use sp_core::{Pair, Public, ed25519, sr25519};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::env;

pub type AccountPublic = <Signature as Verify>::Signer;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate a local sudo account ID from seed using the configured crypto scheme.
pub fn get_local_sudo_account_id_from_seed(seed: &str) -> AccountId {
    match env::var("DUNITER_SUDO_ACCOUNT_CRYPTO").as_deref() {
        Ok("sr25519") => get_account_id_from_seed::<sr25519::Public>(seed),
        _ => get_account_id_from_seed::<ed25519::Public>(seed),
    }
}
