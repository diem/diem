// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::views::{
    AccountRoleView, AccountRoleViewV1, AccountRoleViewV2, AccountView, BytesView, ChildVASP,
    TransactionDataView, TransactionView, VMStatusView, VMStatusViewV1, VMStatusViewV2,
};

#[test]
fn test_serialize_account_view() {
    let vasp = ChildVASP {
        parent_vasp_address: BytesView("".to_string()),
    };
    let account = AccountView {
        balances: vec![],
        sequence_number: 1,
        authentication_key: BytesView("authentication_key".to_string()),
        sent_events_key: BytesView("sent_events_key".to_string()),
        received_events_key: BytesView("received_events_key".to_string()),
        delegated_key_rotation_capability: true,
        delegated_withdrawal_capability: true,
        is_frozen: false,
        role: AccountRoleView {
            role: AccountRoleViewV1::ChildVasp(vasp.clone()),
            role_v2: AccountRoleViewV2::ChildVasp(vasp.clone()),
        },
    };

    let expected = r#"{"balances":[],"sequence_number":1,"authentication_key":"authentication_key","sent_events_key":"sent_events_key","received_events_key":"received_events_key","delegated_key_rotation_capability":true,"delegated_withdrawal_capability":true,"is_frozen":false,"role":{"child_vasp":{"parent_vasp_address":""}},"role_v2":{"type":"child_vasp","parent_vasp_address":""}}"#;

    assert_eq!(expected, serde_json::to_string(&account).unwrap().as_str());
}

#[test]
fn test_serialize_transaction_view() {
    let account = TransactionView {
        version: 12,
        transaction: TransactionDataView::WriteSet {},
        hash: "hash".to_string(),
        events: vec![],
        vm_status: VMStatusView {
            vm_status: VMStatusViewV1::Executed {},
            vm_status_v2: VMStatusViewV2::Executed {},
        },
        gas_used: 11,
    };

    let expected = r#"{"version":12,"transaction":{"type":"writeset"},"hash":"hash","events":[],"vm_status":"executed","vm_status_v2":{"type":"executed"},"gas_used":11}"#;
    assert_eq!(expected, serde_json::to_string(&account).unwrap().as_str());
}
