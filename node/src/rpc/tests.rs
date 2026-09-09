use sc_rpc::{
    DenyUnsafe,
    state::{
        STATE_CALL_MAX_DATA_SIZE, STORAGE_KEYS_MAX_COUNT, is_safe_runtime_call,
        validate_runtime_call, validate_storage_keys_count,
    },
};

#[test]
fn public_runtime_calls_are_allowlisted() {
    for method in [
        "Core_version",
        "Metadata_metadata",
        "AccountNonceApi_account_nonce",
        "TransactionPaymentApi_query_info",
        "DuniterAccountApi_estimate_cost",
        // Directly used by Tikka in production.
        "UniversalDividendApi_account_balances",
    ] {
        assert!(is_safe_runtime_call(method));
        assert!(validate_runtime_call(DenyUnsafe::Yes, method, 0).is_ok());
    }

    for method in [
        "Core_execute_block",
        "BlockBuilder_apply_extrinsic",
        "OffchainWorkerApi_offchain_worker",
        "SessionKeys_generate_session_keys",
        "TaggedTransactionQueue_validate_transaction",
    ] {
        assert!(!is_safe_runtime_call(method));
        assert!(validate_runtime_call(DenyUnsafe::Yes, method, 0).is_err());
        assert!(validate_runtime_call(DenyUnsafe::No, method, 0).is_ok());
    }
}

#[test]
fn runtime_call_data_size_is_bounded() {
    assert!(
        validate_runtime_call(DenyUnsafe::Yes, "Core_version", STATE_CALL_MAX_DATA_SIZE).is_ok()
    );
    assert!(
        validate_runtime_call(
            DenyUnsafe::No,
            "Core_execute_block",
            STATE_CALL_MAX_DATA_SIZE + 1,
        )
        .is_err()
    );
}

#[test]
fn legacy_storage_key_results_are_bounded() {
    assert!(validate_storage_keys_count(STORAGE_KEYS_MAX_COUNT).is_ok());
    assert!(validate_storage_keys_count(STORAGE_KEYS_MAX_COUNT + 1).is_err());
}
