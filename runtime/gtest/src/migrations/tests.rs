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

//! Tests for runtime migrations

use super::*;
use crate::Runtime;
use frame_support::traits::OnRuntimeUpgrade;

/// Build a test externalities with the incorrect NextReeval value
/// (simulating the bug that existed at genesis)
#[cfg(test)]
fn build_test_externalities_with_bug() -> sp_io::TestExternalities {
    use sp_runtime::BuildStorage;

    let mut storage = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    // Initialize Universal Dividend with the INCORRECT value
    // (value in seconds instead of milliseconds)
    pallet_universal_dividend::GenesisConfig::<Runtime> {
        first_reeval: Some(1_766_232_000), // WRONG: interpreted as milliseconds = 1970
        first_ud: Some(1_766_232_000_000), // This one is correct (Dec 20, 2025)
        initial_monetary_mass: 0,
        ud: 1000,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    sp_io::TestExternalities::new(storage)
}

/// Build a test externalities with the correct NextReeval value
#[cfg(test)]
fn build_test_externalities_correct() -> sp_io::TestExternalities {
    use sp_runtime::BuildStorage;

    let mut storage = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    // Initialize Universal Dividend with the CORRECT value
    pallet_universal_dividend::GenesisConfig::<Runtime> {
        first_reeval: Some(1_766_232_000_000), // CORRECT: Dec 20, 2025 in milliseconds
        first_ud: Some(1_766_232_000_000),
        initial_monetary_mass: 0,
        ud: 1000,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    sp_io::TestExternalities::new(storage)
}

#[test]
fn test_migration_v1110_fixes_incorrect_next_reeval() {
    build_test_externalities_with_bug().execute_with(|| {
        // Verify initial state: NextReeval is incorrect (in the past)
        let initial_value = pallet_universal_dividend::NextReeval::<Runtime>::get()
            .expect("NextReeval should be set");

        println!("Initial NextReeval: {}", initial_value);

        // The bug: value is 1_766_232_000 (Jan 21, 1970 in milliseconds)
        assert_eq!(
            initial_value, 1_766_232_000,
            "Initial value should be the buggy value"
        );

        // Execute the migration
        let weight = v1110::FixUdReevalDate::<Runtime>::on_runtime_upgrade();
        println!("Migration weight: {:?}", weight);

        // Verify post-migration state: NextReeval is correct
        let updated_value = pallet_universal_dividend::NextReeval::<Runtime>::get()
            .expect("NextReeval should still be set");

        println!("Updated NextReeval: {}", updated_value);

        // The fix: value should now be 1_766_232_000_000 (Dec 20, 2025 in milliseconds)
        assert_eq!(
            updated_value, 1_766_232_000_000,
            "NextReeval should be corrected to the proper milliseconds value"
        );

        // Verify the value actually changed
        assert_ne!(
            initial_value, updated_value,
            "NextReeval value should have changed"
        );
    });
}

#[test]
fn test_migration_v1110_is_idempotent() {
    build_test_externalities_with_bug().execute_with(|| {
        // Run migration first time
        v1110::FixUdReevalDate::<Runtime>::on_runtime_upgrade();

        let first_run_value = pallet_universal_dividend::NextReeval::<Runtime>::get()
            .expect("NextReeval should be set");

        // Run migration second time (should be idempotent)
        v1110::FixUdReevalDate::<Runtime>::on_runtime_upgrade();

        let second_run_value = pallet_universal_dividend::NextReeval::<Runtime>::get()
            .expect("NextReeval should be set");

        // Both runs should result in the same value
        assert_eq!(
            first_run_value, second_run_value,
            "Migration should be idempotent"
        );
        assert_eq!(
            second_run_value, 1_766_232_000_000,
            "Value should still be correct"
        );
    });
}

#[test]
fn test_migration_v1110_with_already_correct_value() {
    build_test_externalities_correct().execute_with(|| {
        // Initial value is already correct
        let initial_value = pallet_universal_dividend::NextReeval::<Runtime>::get()
            .expect("NextReeval should be set");

        assert_eq!(
            initial_value, 1_766_232_000_000,
            "Initial value should already be correct"
        );

        // Execute migration
        v1110::FixUdReevalDate::<Runtime>::on_runtime_upgrade();

        // Verify value is still correct (unchanged)
        let updated_value = pallet_universal_dividend::NextReeval::<Runtime>::get()
            .expect("NextReeval should still be set");

        assert_eq!(
            updated_value, 1_766_232_000_000,
            "Value should remain correct"
        );
        assert_eq!(
            initial_value, updated_value,
            "Value should not change if already correct"
        );
    });
}

#[cfg(feature = "try-runtime")]
#[test]
fn test_migration_v1110_try_runtime_checks() {
    build_test_externalities_with_bug().execute_with(|| {
        // Pre-upgrade check
        let state =
            v1110::FixUdReevalDate::<Runtime>::pre_upgrade().expect("Pre-upgrade should succeed");

        // Execute migration
        v1110::FixUdReevalDate::<Runtime>::on_runtime_upgrade();

        // Post-upgrade check
        v1110::FixUdReevalDate::<Runtime>::post_upgrade(state)
            .expect("Post-upgrade should succeed and verify the migration");
    });
}

#[cfg(feature = "try-runtime")]
#[test]
fn test_migration_v1110_post_upgrade_detects_incorrect_value() {
    build_test_externalities_with_bug().execute_with(|| {
        // Pre-upgrade check
        let state =
            v1110::FixUdReevalDate::<Runtime>::pre_upgrade().expect("Pre-upgrade should succeed");

        // DON'T execute the migration - simulate it failing

        // Post-upgrade check should fail because value wasn't updated
        let result = v1110::FixUdReevalDate::<Runtime>::post_upgrade(state);
        assert!(
            result.is_err(),
            "Post-upgrade should fail if migration didn't run"
        );
    });
}
