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

// Timestamp
parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
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
    pub const ConfirmPeriod: BlockNumber = DAYS;
    pub const MaxInactivityPeriod: BlockNumber = 73 * DAYS;
    pub const MaxNoRightPeriod: BlockNumber = 73 * DAYS;
    pub const IdtyRenewablePeriod: BlockNumber = 12 * DAYS;
    pub const ValidationPeriod: BlockNumber = 73 * DAYS;
}

// Certification
pub const MIN_STRONG_CERT_FOR_UD: u32 = 5;
pub const MIN_STRONG_CERT_FOR_STRONG_CERT: u32 = 5;
parameter_types! {
    pub const CertPeriod: BlockNumber = DAYS;
    pub const MaxByIssuer: u8 = 100;
    pub const StrongCertRenewablePeriod: BlockNumber = 12 * DAYS;
    pub const ValidityPeriod: BlockNumber = 146 * DAYS;
}

// Universal dividend
parameter_types! {
    pub const SquareMoneyGrowthRate: Permill = Permill::from_parts(2_381_440); // 0.002_381_440 = 0.0488^2
    pub const UdCreationPeriod: BlockNumber = DAYS;
    pub const UdReevalPeriod: Balance = 7;
    pub const UdReevalPeriodInBlocks: BlockNumber = 100800; // 86400 *7 / 6
}
