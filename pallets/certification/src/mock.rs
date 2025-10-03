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

use crate::{self as pallet_certification};
use frame_support::{
    derive_impl, parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;
pub type IdtyIndex = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        DefaultCertification: pallet_certification,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
    type Block = Block;
    type BlockHashCount = BlockHashCount;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = u64;
    type PalletInfo = PalletInfo;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const MaxByIssuer: u32 = 4;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 2;
    pub const CertPeriod: u64 = 2;
    pub const ValidityPeriod: u64 = 10;
}

impl pallet_certification::Config for Test {
    type CertPeriod = CertPeriod;
    type CheckCertAllowed = ();
    type IdtyAttr = ();
    type IdtyIndex = IdtyIndex;
    type MaxByIssuer = MaxByIssuer;
    type MinReceivedCertToBeAbleToIssueCert = MinReceivedCertToBeAbleToIssueCert;
    type OnNewcert = ();
    type OnRemovedCert = ();
    type ValidityPeriod = ValidityPeriod;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(
    gen_conf: pallet_certification::GenesisConfig<Test>,
) -> sp_io::TestExternalities {
    RuntimeGenesisConfig {
        system: SystemConfig::default(),
        default_certification: gen_conf,
    }
    .build_storage()
    .unwrap()
    .into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        DefaultCertification::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        DefaultCertification::on_initialize(System::block_number());
    }
}
