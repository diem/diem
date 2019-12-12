// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_get_latest_state_root_response(resp in any::<GetLatestStateRootResponse>()) {
        assert_protobuf_encode_decode::<crate::proto::storage::GetLatestStateRootResponse, GetLatestStateRootResponse>(&resp);
    }

    #[test]
    fn test_save_transactions_request(req in any::<SaveTransactionsRequest>()) {
        assert_protobuf_encode_decode::<crate::proto::storage::SaveTransactionsRequest, SaveTransactionsRequest>(&req);
    }

    #[test]
    fn test_get_transactions_request(req in any::<GetTransactionsRequest>()) {
        assert_protobuf_encode_decode::<crate::proto::storage::GetTransactionsRequest, GetTransactionsRequest>(&req);
    }

    #[test]
    fn test_get_transactions_response(resp in any::<GetTransactionsResponse>()) {
        assert_protobuf_encode_decode::<crate::proto::storage::GetTransactionsResponse, GetTransactionsResponse>(&resp);
    }

    #[test]
    fn test_startup_info(startup_info in any::<StartupInfo>()) {
        assert_protobuf_encode_decode::<crate::proto::storage::StartupInfo, StartupInfo>(&startup_info);
    }

    #[test]
    fn test_get_startup_info_response(res in any::<GetStartupInfoResponse>()) {
        assert_protobuf_encode_decode::<crate::proto::storage::GetStartupInfoResponse, GetStartupInfoResponse>(&res);
    }
}
