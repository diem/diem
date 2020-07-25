// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::views::{
    AccountRoleView, AccountView, BytesView, TransactionDataView, TransactionView, VMStatusView,
};

#[test]
fn test_serialize_account_view() {
    let account = AccountView {
        balances: vec![],
        sequence_number: 1,
        authentication_key: BytesView("authentication_key".to_string()),
        sent_events_key: BytesView("sent_events_key".to_string()),
        received_events_key: BytesView("received_events_key".to_string()),
        delegated_key_rotation_capability: true,
        delegated_withdrawal_capability: true,
        is_frozen: false,
        role: AccountRoleView::Unknown {},
    };

    let expected = "{\"balances\":[],\"sequence_number\":1,\"authentication_key\":\"authentication_key\",\"sent_events_key\":\"sent_events_key\",\"received_events_key\":\"received_events_key\",\"delegated_key_rotation_capability\":true,\"delegated_withdrawal_capability\":true,\"is_frozen\":false,\"role\":{\"type\":\"unknown\"}}";

    assert_eq!(expected, serde_json::to_string(&account).unwrap().as_str());
}

#[test]
fn test_serialize_transaction_view() {
    let account = TransactionView {
        version: 12,
        transaction: TransactionDataView::WriteSet {},
        hash: "hash".to_string(),
        events: vec![],
        vm_status: VMStatusView::Executed {},
        gas_used: 11,
    };

    let expected = "{\"version\":12,\"transaction\":{\"type\":\"writeset\"},\"hash\":\"hash\",\"events\":[],\"vm_status\":{\"type\":\"executed\"},\"gas_used\":11}";
    assert_eq!(expected, serde_json::to_string(&account).unwrap().as_str());
}
