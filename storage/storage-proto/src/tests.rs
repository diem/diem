// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;

macro_rules! test_conversion {
    ($test_name: ident, $rust_type: ident $(,)?) => {
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10))]

            #[test]
            fn $test_name(object in any::<$rust_type>()) {
                type ProtoType = crate::proto::storage::$rust_type;
                assert_protobuf_encode_decode::<ProtoType, $rust_type>(&object);
            }
        }
    };
}

test_conversion!(
    test_get_latest_state_root_response,
    GetLatestStateRootResponse,
);
test_conversion!(
    test_get_latest_account_state_request,
    GetLatestAccountStateRequest,
);
test_conversion!(
    test_get_latest_account_state_response,
    GetLatestAccountStateResponse,
);
test_conversion!(
    test_get_account_state_with_proof_by_version_request,
    GetAccountStateWithProofByVersionRequest,
);
test_conversion!(
    test_get_account_state_with_proof_by_version_response,
    GetAccountStateWithProofByVersionResponse,
);
test_conversion!(test_save_transactions_request, SaveTransactionsRequest);
test_conversion!(test_get_transactions_request, GetTransactionsRequest);
test_conversion!(test_get_transactions_response, GetTransactionsResponse);
test_conversion!(test_tree_state, TreeState);
test_conversion!(test_startup_info, StartupInfo);
test_conversion!(test_get_startup_info_response, GetStartupInfoResponse);
test_conversion!(
    test_get_epoch_change_ledger_infos_request,
    GetEpochChangeLedgerInfosRequest,
);
test_conversion!(test_backup_account_state_request, BackupAccountStateRequest);
test_conversion!(
    test_backup_account_state_response,
    BackupAccountStateResponse,
);
test_conversion!(
    test_get_account_state_range_proof_request,
    GetAccountStateRangeProofRequest,
);
test_conversion!(
    test_get_account_state_range_proof_response,
    GetAccountStateRangeProofResponse,
);
test_conversion!(test_backup_transaction_request, BackupTransactionRequest);
test_conversion!(test_backup_transaction_response, BackupTransactionResponse);
test_conversion!(
    test_backup_transaction_info_request,
    BackupTransactionInfoRequest,
);
test_conversion!(
    test_backup_transaction_info_response,
    BackupTransactionInfoResponse,
);
