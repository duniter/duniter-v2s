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

//! # Migration v1110: Fix Universal Dividend Revaluation Date
//!
//! ## Context
//!
//! The gtest network was launched with an incorrect `first_ud_reeval` value in genesis.
//! The value was set to `1766232000` (interpreted as milliseconds = Jan 21, 1970)
//! instead of `1766232000000` (Dec 20, 2025).
//!
//! This caused the UD to be reevaluated every day instead of every 6 months,
//! because the condition `current_time >= next_reeval` was always true.
//!
//! ## Fix
//!
//! This migration sets `NextReeval` to the correct value: 1766232000000 milliseconds
//! (December 20, 2025 at 13:00:00 UTC).

extern crate alloc;

use core::marker::PhantomData;
use frame_support::{
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// Fix the NextReeval date for Universal Dividend
pub struct FixUdReevalDate<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for FixUdReevalDate<T>
where
    T: pallet_universal_dividend::Config<Moment = u64> + frame_system::Config,
{
    fn on_runtime_upgrade() -> Weight {
        log::info!("🔧 Migration v1110: Fixing Universal Dividend NextReeval date");

        // The correct value: December 20, 2025 at 13:00:00 UTC in milliseconds
        const CORRECT_NEXT_REEVAL_MS: u64 = 1_766_232_000_000;

        // Get the current (incorrect) value
        let old_value = pallet_universal_dividend::NextReeval::<T>::get();
        log::info!(
            "  Old NextReeval value: {:?}",
            old_value.unwrap_or_default()
        );

        // Set the correct value
        pallet_universal_dividend::NextReeval::<T>::put(CORRECT_NEXT_REEVAL_MS);

        log::info!("  New NextReeval value: {}", CORRECT_NEXT_REEVAL_MS);
        log::info!("✅ Migration v1110: NextReeval successfully updated");

        // Return weight: 1 read + 1 write
        T::DbWeight::get().reads_writes(1, 1)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<alloc::vec::Vec<u8>, TryRuntimeError> {
        use codec::Encode;

        log::info!("⚙️  Pre-upgrade v1110: Checking current NextReeval");

        let old_value = pallet_universal_dividend::NextReeval::<T>::get();
        log::info!("  Current NextReeval: {:?}", old_value);

        // Return the old value encoded so we can verify it changed
        Ok(old_value.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: alloc::vec::Vec<u8>) -> Result<(), TryRuntimeError> {
        use codec::Decode;

        log::info!("✅ Post-upgrade v1110: Verifying NextReeval was updated");

        const CORRECT_NEXT_REEVAL_MS: u64 = 1_766_232_000_000;

        // Decode the old value
        let old_value = Option::<u64>::decode(&mut &state[..])
            .map_err(|_| TryRuntimeError::Other("Failed to decode old NextReeval value"))?;

        // Get the new value
        let new_value = pallet_universal_dividend::NextReeval::<T>::get()
            .ok_or(TryRuntimeError::Other("NextReeval should be set"))?;

        // Verify the value was updated correctly
        if new_value != CORRECT_NEXT_REEVAL_MS {
            log::error!(
                "  ❌ NextReeval not correctly set! Expected: {}, Got: {}",
                CORRECT_NEXT_REEVAL_MS,
                new_value
            );
            return Err(TryRuntimeError::Other(
                "NextReeval was not set to the correct value",
            ));
        }

        // Verify the value actually changed
        if let Some(old) = old_value {
            if old == new_value {
                log::warn!("  ⚠️  NextReeval value did not change (was already correct?)");
            } else {
                log::info!(
                    "  ✅ NextReeval successfully changed from {} to {}",
                    old,
                    new_value
                );
            }
        }

        log::info!("✅ Post-upgrade v1110: Migration verification successful");
        Ok(())
    }
}
