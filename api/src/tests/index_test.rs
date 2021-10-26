// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tests::new_test_context;
use serde_json::json;

#[tokio::test]
async fn test_get_ledger_info() {
    let context = new_test_context();
    let ledger_info = context.get_latest_ledger_info();
    let resp = context.get("/").await;

    let expected = json!({
        "chain_id": 4,
        "ledger_version": ledger_info.version().to_string(),
        "ledger_timestamp": ledger_info.timestamp().to_string(),
    });

    assert_eq!(expected, resp);
}

#[tokio::test]
async fn test_returns_not_found_for_the_invalid_path() {
    let context = new_test_context();
    let resp = context.expect_status_code(404).get("/invalid_path").await;
    assert_eq!(json!({"code": 404, "message": "Not Found"}), resp)
}

#[tokio::test]
async fn test_return_bad_request_if_method_not_allowed() {
    let context = new_test_context();
    let resp = context
        .expect_status_code(400)
        .post("/accounts/0x1/resources", json!({}))
        .await;

    let expected = json!({
        "code": 400,
        "message": "Method not allowed or request body is invalid.",
    });

    assert_eq!(expected, resp);
}
