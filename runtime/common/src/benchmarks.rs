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

#![cfg(feature = "runtime-benchmarks")]

#[macro_export]
macro_rules! benchmarks_config {
    () => {
        #[macro_use]
        extern crate frame_benchmarking;
        pub use pallet_collective::RawOrigin;

        type WorstOrigin = RawOrigin<AccountId, TechnicalCommitteeInstance>;

        mod benches {
            define_benchmarks!(
                [pallet_certification, Certification]
                [pallet_distance, Distance]
                [pallet_oneshot_account, OneshotAccount]
                [pallet_universal_dividend, UniversalDividend]
                [pallet_provide_randomness, ProvideRandomness]
                [pallet_upgrade_origin, UpgradeOrigin]
                [pallet_duniter_account, Account]
                [pallet_quota, Quota]
                [pallet_identity, Identity]
                [pallet_membership, Membership]
                [pallet_smith_members, SmithMembers]
                [pallet_authority_members, AuthorityMembers]
                // Substrate
                [frame_system_extensions, SystemExtensionsBench::<Runtime>]
                [pallet_balances, Balances]
                [frame_benchmarking::baseline, Baseline::<Runtime>]
                [pallet_collective, TechnicalCommittee]
                [pallet_session, SessionBench::<Runtime>]
                [pallet_im_online, ImOnline]
                [pallet_sudo, Sudo]
                [pallet_multisig, Multisig]
                [pallet_preimage, Preimage]
                [pallet_proxy, Proxy]
                [pallet_scheduler, Scheduler]
                [frame_system, SystemBench::<Runtime>]
                [pallet_timestamp, Timestamp]
                [pallet_transaction_payment, TransactionPayment]
                [pallet_treasury, Treasury]
                [pallet_utility, Utility]
            );
        }
    };
}
