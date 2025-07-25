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

// This file is not formatted automatically due to the nested macro definition.
// It can be formatted by commenting out the last line, the first five lines,
// and then running `rustfmt` on `runtime/common/src/apis.rs`.
#[macro_export]
macro_rules! runtime_apis {
    ($($custom:tt)*) => {
        impl_runtime_apis! {
            $($custom)*

impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
    fn authorities() -> Vec<sp_authority_discovery::AuthorityId> {
        AuthorityDiscovery::authorities()
    }
}

impl sp_consensus_babe::BabeApi<Block> for Runtime {
    fn configuration() -> sp_consensus_babe::BabeConfiguration {
        // The choice of `c` parameter (where `1 - c` represents the
        // probability of a slot being empty), is done in accordance to the
        // slot duration and expected target block time, for safely
        // resisting network delays of maximum two seconds.
        // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
        use frame_support::traits::Get as _;
        sp_consensus_babe::BabeConfiguration {
            slot_duration: Babe::slot_duration(),
            epoch_length: EpochDuration::get(),
            c: BABE_GENESIS_EPOCH_CONFIG.c,
            authorities: Babe::authorities().to_vec(),
            randomness: Babe::randomness(),
            allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
        }
    }

    fn current_epoch_start() -> sp_consensus_babe::Slot {
        Babe::current_epoch_start()
    }

    fn current_epoch() -> sp_consensus_babe::Epoch {
        Babe::current_epoch()
    }

    fn next_epoch() -> sp_consensus_babe::Epoch {
        Babe::next_epoch()
    }

    fn generate_key_ownership_proof(
        _slot: sp_consensus_babe::Slot,
        authority_id: sp_consensus_babe::AuthorityId,
    ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
        use codec::Encode;

        Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
            .map(|p| p.encode())
            .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
    }

    fn submit_report_equivocation_unsigned_extrinsic(
        equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
        key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
    ) -> Option<()> {
        let key_owner_proof = key_owner_proof.decode()?;

        Babe::submit_unsigned_equivocation_report(equivocation_proof, key_owner_proof)
    }
}

impl sp_api::Core<Block> for Runtime {
    fn version() -> RuntimeVersion {
        VERSION
    }

    fn execute_block(block: Block) {
        Executive::execute_block(block)
    }

    fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
        Executive::initialize_block(header)
    }
}

impl sp_api::Metadata<Block> for Runtime {
    fn metadata() -> OpaqueMetadata {
        OpaqueMetadata::new(Runtime::metadata().into())
    }

    fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
        Runtime::metadata_at_version(version)
    }

    fn metadata_versions() -> Vec<u32> {
        Runtime::metadata_versions()
    }
}

impl sp_block_builder::BlockBuilder<Block> for Runtime {
    fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
        Executive::apply_extrinsic(extrinsic)
    }

    fn finalize_block() -> <Block as BlockT>::Header {
        Executive::finalize_block()
    }

    fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
        data.create_extrinsics()
    }

    fn check_inherents(
        block: Block,
        data: sp_inherents::InherentData,
    ) -> sp_inherents::CheckInherentsResult {
        data.check_extrinsics(&block)
    }
}

impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
    fn validate_transaction(
        source: TransactionSource,
        tx: <Block as BlockT>::Extrinsic,
        block_hash: <Block as BlockT>::Hash,
    ) -> TransactionValidity {
        // Filtered calls should not enter the tx pool.
        if !<Runtime as frame_system::Config>::BaseCallFilter::contains(&tx.function) {
            return sp_runtime::transaction_validity::InvalidTransaction::Call.into();
        }
        Executive::validate_transaction(source, tx, block_hash)
    }
}

impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
    fn offchain_worker(header: &<Block as BlockT>::Header) {
        Executive::offchain_worker(header)
    }
}

impl sp_session::SessionKeys<Block> for Runtime {
    fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
        opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
    }

    fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
        opaque::SessionKeys::generate(seed)
    }
}

impl fg_primitives::GrandpaApi<Block> for Runtime {
    fn grandpa_authorities() -> GrandpaAuthorityList {
        Grandpa::grandpa_authorities()
    }

    fn current_set_id() -> fg_primitives::SetId {
        Grandpa::current_set_id()
    }

    fn submit_report_equivocation_unsigned_extrinsic(
        _equivocation_proof: fg_primitives::EquivocationProof<
            <Block as BlockT>::Hash,
            NumberFor<Block>,
        >,
        _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
    ) -> Option<()> {
        None
    }

    fn generate_key_ownership_proof(
        _set_id: fg_primitives::SetId,
        _authority_id: GrandpaId,
    ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
        // NOTE: this is the only implementation possible since we've
        // defined our key owner proof type as a bottom type (i.e. a type
        // with no values).
        None
    }
}

impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
    fn account_nonce(account: AccountId) -> Index {
        System::account_nonce(account)
    }
}

impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
    fn query_info(
        uxt: <Block as BlockT>::Extrinsic,
        len: u32,
    ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
        TransactionPayment::query_info(uxt, len)
    }

    fn query_fee_details(
        uxt: <Block as BlockT>::Extrinsic,
        len: u32,
    ) -> pallet_transaction_payment::FeeDetails<Balance> {
        TransactionPayment::query_fee_details(uxt, len)
    }

    fn query_weight_to_fee(weight: Weight) -> Balance {
        TransactionPayment::weight_to_fee(weight)
    }

    fn query_length_to_fee(length: u32) -> Balance {
        TransactionPayment::length_to_fee(length)
    }
}

impl pallet_duniter_account::DuniterAccountApi<Block, Balance> for Runtime {
    fn estimate_cost(
        uxt: <Block as BlockT>::Extrinsic,
    ) -> pallet_duniter_account::EstimatedCost<Balance> {
        pallet_duniter_account::Pallet::<Runtime>::estimate_cost(uxt)
    }
}

impl pallet_universal_dividend::UniversalDividendApi<Block, AccountId, Balance> for Runtime {
    fn account_balances(account: AccountId) -> pallet_universal_dividend::AccountBalances<Balance> {
        UniversalDividend::account_balances(&account)
    }
}

impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
    fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
        frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
    }

    fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
        frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
    }

    fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
        vec![]
    }
}

#[cfg(feature = "try-runtime")]
impl frame_try_runtime::TryRuntime<Block> for Runtime {
    fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
        let weight = Executive::try_runtime_upgrade(checks).unwrap();
        (weight, BlockWeights::get().max_block)
    }

    fn execute_block(
        block: Block,
        state_root_check: bool,
        signature_check: bool,
        select: frame_try_runtime::TryStateSelect,
    ) -> Weight {
        Executive::try_execute_block(block, state_root_check, signature_check, select)
            .expect("execute-block failed")
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl frame_benchmarking::Benchmark<Block> for Runtime {
    fn benchmark_metadata(
        extra: bool,
    ) -> (
        Vec<frame_benchmarking::BenchmarkList>,
        Vec<frame_support::traits::StorageInfo>,
    ) {
        use frame_benchmarking::{list_benchmark, BenchmarkList, Benchmarking};
        use frame_support::traits::StorageInfoTrait;

        use frame_benchmarking::baseline::Pallet as Baseline;
        use frame_system_benchmarking::{
            extensions::Pallet as SystemExtensionsBench, Pallet as SystemBench,
        };
        use pallet_session_benchmarking::Pallet as SessionBench;

        let mut list = Vec::<BenchmarkList>::new();
        list_benchmarks!(list, extra);

        let storage_info = AllPalletsWithSystem::storage_info();
        return (list, storage_info);
    }

    fn dispatch_benchmark(
        config: frame_benchmarking::BenchmarkConfig,
    ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, scale_info::prelude::string::String> {
        use frame_benchmarking::{baseline::Pallet as Baseline, BenchmarkBatch, Benchmarking};
        use frame_support::traits::{TrackedStorageKey, WhitelistedStorageKeys};
        use frame_system_benchmarking::{
            extensions::Pallet as SystemExtensionsBench, Pallet as SystemBench,
        };
        use pallet_session_benchmarking::Pallet as SessionBench;

        impl pallet_session_benchmarking::Config for Runtime {}
        impl frame_system_benchmarking::Config for Runtime {}
        impl frame_benchmarking::baseline::Config for Runtime {}

        /*let whitelist: Vec<TrackedStorageKey> = vec![
        // Block Number
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
        // Total Issuance
        hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
        // Execution Phase
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
        // Event Count
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
        // System Events
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
        // Treasury Account
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
        ];*/

        let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();
        let mut batches = Vec::<BenchmarkBatch>::new();
        let params = (&config, &whitelist);
        add_benchmarks!(params, batches);

        if batches.is_empty() {
            return Err("Benchmark not found for this pallet.".into());
        }
        Ok(batches)
    }
}
}};}
