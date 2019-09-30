// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_proto_conv::test_helper::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_save_transactions_request(req in any::<SaveTransactionsRequest>()) {
        assert_protobuf_encode_decode(&req);
    }

    #[test]
    fn test_get_transactions_request(req in any::<GetTransactionsRequest>()) {
        assert_protobuf_encode_decode(&req);
    }

    #[test]
    fn test_get_transactions_response(resp in any::<GetTransactionsResponse>()) {
        assert_protobuf_encode_decode(&resp);
    }

    #[test]
    fn test_startup_info(startup_info in any::<StartupInfo>()) {
        assert_protobuf_encode_decode(&startup_info);
    }

    #[test]
    fn test_get_startup_info_response(res in any::<GetStartupInfoResponse>()) {
        assert_protobuf_encode_decode(&res);
    }
}
