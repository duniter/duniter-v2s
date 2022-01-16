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
use frame_support::weights::constants::WEIGHT_PER_SECOND;
use sp_arithmetic::Permill;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 42;
}

// Balances
frame_support::parameter_types! {
    pub const ExistentialDeposit: Balance = 500;
    pub const MaxLocks: u32 = 50;
}

// Consensus
parameter_types! {
    pub const MaxAuthorities: u32 = 10;
}

// Transaction payment
frame_support::parameter_types! {
    pub const TransactionByteFee: Balance = 0;
}

// Identity
pub const IDTY_CREATE_PERIOD: BlockNumber = 100;
frame_support::parameter_types! {
    pub const ConfirmPeriod: BlockNumber = 40;
    pub const FirstIssuableOn: BlockNumber = 20;
    pub const IdtyCreationPeriod: BlockNumber = 50;
    pub const IdtyRenewablePeriod: BlockNumber = 50;
    pub const MaxInactivityPeriod: BlockNumber = 1_000;
    pub const MaxNoRightPeriod: BlockNumber = 1_000;
    pub const ValidationPeriod: BlockNumber = 500;
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
    pub const UdCreationPeriod: BlockNumber = 10;
    pub const UdFirstReeval: BlockNumber = 100;
    pub const UdReevalPeriod: Balance = 10;
    pub const UdReevalPeriodInBlocks: BlockNumber = 20 * 10;
}

// Multisig
parameter_types! {
    pub const DepositBase: Balance = 1000;
    pub const DepositFactor: Balance = 10;
    pub const MaxSignatories: u16 = 5;
}
