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
    pub const ConfirmPeriod: BlockNumber = 12 * HOURS;
    pub const MaxInactivityPeriod: BlockNumber = YEARS;
    pub const MaxNoRightPeriod: BlockNumber = YEARS;
    pub const IdtyRenewablePeriod: BlockNumber = 6 * MONTHS;
    pub const ValidationPeriod: BlockNumber = 2 * MONTHS;
}

// Certification
pub const MIN_STRONG_CERT_FOR_UD: u32 = 2;
pub const MIN_STRONG_CERT_FOR_STRONG_CERT: u32 = 3;
parameter_types! {
    pub const CertPeriod: BlockNumber = 15;
    pub const MaxByIssuer: u8 = 100;
    pub const StrongCertRenewablePeriod: BlockNumber = 50;//6 * MONTHS;
    pub const ValidityPeriod: BlockNumber = 200;//2 * YEARS;
}

// Universal dividend
parameter_types! {
    pub const SquareMoneyGrowthRate: Permill = Permill::one();
    pub const UdCreationPeriod: BlockNumber = 20;
    pub const UdReevalPeriod: Balance = 10;
    pub const UdReevalPeriodInBlocks: BlockNumber = 20 * 10;
}
