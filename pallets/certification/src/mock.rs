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

use crate::{self as pallet_certification};
use frame_support::{
    parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type AccountId = u64;
type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Test>;
pub type IdtyIndex = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        DefaultCertification: pallet_certification::{Pallet, Call, Event<T>, Storage, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub struct EnsureRoot;
impl frame_support::traits::EnsureOrigin<(Origin, IdtyIndex, IdtyIndex)> for EnsureRoot {
    type Success = ();

    fn try_origin(
        o: (Origin, IdtyIndex, IdtyIndex),
    ) -> Result<Self::Success, (Origin, IdtyIndex, IdtyIndex)> {
        match o.0.clone().into() {
            Ok(system::RawOrigin::Root) => Ok(()),
            _ => Err(o),
        }
    }
}

parameter_types! {
    pub const MaxByIssuer: u32 = 3;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 2;
    pub const RenewablePeriod: BlockNumber = 4;
    pub const CertPeriod: u64 = 2;
    pub const ValidityPeriod: u64 = 10;
}

impl pallet_certification::Config for Test {
    type AddCertOrigin = EnsureRoot;
    type CertPeriod = CertPeriod;
    type DelCertOrigin = EnsureRoot;
    type Event = Event;
    type IdtyIndex = IdtyIndex;
    type MaxByIssuer = MaxByIssuer;
    type MinReceivedCertToBeAbleToIssueCert = MinReceivedCertToBeAbleToIssueCert;
    type OnNewcert = ();
    type OnRemovedCert = ();
    type CertRenewablePeriod = RenewablePeriod;
    type ValidityPeriod = ValidityPeriod;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(
    gen_conf: pallet_certification::GenesisConfig<Test>,
) -> sp_io::TestExternalities {
    GenesisConfig {
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
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        DefaultCertification::on_initialize(System::block_number());
    }
}
