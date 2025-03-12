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

use super::*;
use crate::{self as pallet_identity};
use frame_support::{
    derive_impl, parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::{Pair, H256};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, MultiSignature, MultiSigner,
};
use sp_state_machine::BasicExternalities;
use std::sync::Arc;

type Block = frame_system::mocking::MockBlock<Test>;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

fn account(id: u8) -> AccountId {
    let pair = sp_core::sr25519::Pair::from_seed(&[id; 32]);
    MultiSigner::Sr25519(pair.public()).into_account()
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Identity: pallet_identity,
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
    pub const ChangeOwnerKeyPeriod: u64 = 10;
    pub const ConfirmPeriod: u64 = 2;
    pub const ValidationPeriod: u64 = 3;
    pub const AutorevocationPeriod: u64 = 5;
    pub const DeletionPeriod: u64 = 7;
    pub const IdtyCreationPeriod: u64 = 3;
}

pub struct IdtyNameValidatorTestImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorTestImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() < 16
    }
}

impl pallet_identity::Config for Test {
    type AccountLinker = ();
    type AutorevocationPeriod = AutorevocationPeriod;
    type ChangeOwnerKeyPeriod = ChangeOwnerKeyPeriod;
    type CheckAccountWorthiness = ();
    type CheckIdtyCallAllowed = ();
    type ConfirmPeriod = ConfirmPeriod;
    type DeletionPeriod = DeletionPeriod;
    type IdtyCreationPeriod = IdtyCreationPeriod;
    type IdtyData = ();
    type IdtyIndex = u64;
    type IdtyNameValidator = IdtyNameValidatorTestImpl;
    type OnNewIdty = ();
    type OnRemoveIdty = ();
    type OwnerKeyChangePermission = ();
    type RuntimeEvent = RuntimeEvent;
    type Signature = Signature;
    type Signer = AccountPublic;
    type ValidationPeriod = ValidationPeriod;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(gen_conf: pallet_identity::GenesisConfig<Test>) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    gen_conf.assimilate_storage(&mut t).unwrap();

    BasicExternalities::execute_with_storage(&mut t, || {
        frame_system::Pallet::<Test>::inc_providers(&account(2));
        frame_system::Pallet::<Test>::inc_providers(&account(3));
    });

    let keystore = MemoryKeystore::new();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.register_extension(KeystoreExt(Arc::new(keystore)));
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Identity::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
    }
}
