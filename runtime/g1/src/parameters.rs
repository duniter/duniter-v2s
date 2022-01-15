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

use common_runtime::constants::*;
use common_runtime::{Balance, BlockNumber};
use frame_support::parameter_types;
use sp_arithmetic::Permill;

// Authority discovery
parameter_types! {
    pub const MaxAuthorities: u32 = 100;
}

// Timestamp
parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

// Babe
pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = 4 * HOURS;
parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 = 168 * EpochDuration::get();
}

// Balances
frame_support::parameter_types! {
    pub const ExistentialDeposit: Balance = 500;
    pub const MaxLocks: u32 = 50;
}

// Transaction payment
frame_support::parameter_types! {
    pub const TransactionByteFee: Balance = 0;
}

// Identity
pub const IDTY_CREATE_PERIOD: BlockNumber = 100;
frame_support::parameter_types! {
    pub const ConfirmPeriod: BlockNumber = 3 * DAYS;
    pub const MaxInactivityPeriod: BlockNumber = YEARS;
    pub const MaxNoRightPeriod: BlockNumber = YEARS;
    pub const IdtyRenewablePeriod: BlockNumber = 6 * MONTHS;
    pub const ValidationPeriod: BlockNumber = YEARS;
}

// Certification
pub const MIN_STRONG_CERT_FOR_UD: u32 = 5;
pub const MIN_STRONG_CERT_FOR_STRONG_CERT: u32 = 5;
parameter_types! {
    pub const CertPeriod: BlockNumber = 5 * DAYS;
    pub const MaxByIssuer: u8 = 100;
    pub const StrongCertRenewablePeriod: BlockNumber = 6 * MONTHS;
    pub const ValidityPeriod: BlockNumber = 2 * YEARS;
}

// Universal dividend
parameter_types! {
    pub const SquareMoneyGrowthRate: Permill = Permill::from_parts(2_381_440); // 0.002_381_440 = 0.0488^2
    pub const UdCreationPeriod: BlockNumber = DAYS;
    // TODO: this value will depend on the date of the migration
    pub const UdFirstReeval: BlockNumber = 45 * DAYS;
    pub const UdReevalPeriod: Balance = 182;
    pub const UdReevalPeriodInBlocks: BlockNumber = 2_620_800; // 86400 * 182 / 6
}

// Multisig
parameter_types! {
    pub const DepositBase: Balance = 1000;
    pub const DepositFactor: Balance = 10;
    pub const MaxSignatories: u16 = 5;
}
