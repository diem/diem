// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines constants for enum type values
//! See the following Diem JSON-RPC response type documents for more details:
//! * https://github.com/diem/diem/blob/master/json-rpc/docs/type_account.md#type-account
//! * https://github.com/diem/diem/blob/master/json-rpc/docs/type_event.md#event-data
//! * https://github.com/diem/diem/blob/master/json-rpc/docs/type_transaction.md#type-vmstatus
//! * https://github.com/diem/diem/blob/master/json-rpc/docs/type_transaction.md#type-transactiondata

// AccountRole#type field values
pub const ACCOUNT_ROLE_UNKNOWN: &str = "unknown";
pub const ACCOUNT_ROLE_CHILD_VASP: &str = "child_vasp";
pub const ACCOUNT_ROLE_PARENT_VASP: &str = "parent_vasp";
pub const ACCOUNT_ROLE_DESIGNATED_DEALER: &str = "designated_dealer";

// EventData#type field values
pub const EVENT_DATA_UNKNOWN: &str = "unknown";
pub const EVENT_DATA_BURN: &str = "burn";
pub const EVENT_DATA_CANCEL_BURN: &str = "cancelburn";
pub const EVENT_DATA_MINT: &str = "mint";
pub const EVENT_DATA_TO_XDX_EXCHANGE_RATE_UPDATE: &str = "to_xdx_exchange_rate_update";
pub const EVENT_DATA_PREBURN: &str = "preburn";
pub const EVENT_DATA_RECEIVED_PAYMENT: &str = "receivedpayment";
pub const EVENT_DATA_SENT_PAYMENT: &str = "sentpayment";
pub const EVENT_DATA_NEW_EPOCH: &str = "newepoch";
pub const EVENT_DATA_NEW_BLOCK: &str = "newblock";
pub const EVENT_DATA_RECEIVED_MINT: &str = "receivedmint";
pub const EVENT_DATA_COMPLIANCE_KEY_ROTATION: &str = "compliancekeyrotation";
pub const EVENT_DATA_BASE_URL_ROTATION: &str = "baseurlrotation";
pub const EVENT_DATA_CREATE_ACCOUNT: &str = "createaccount";
pub const EVENT_DATA_ADMIN_TRANSACTION: &str = "admintransaction";

// VMStatus#type field values
pub const VM_STATUS_EXECUTED: &str = "executed";
pub const VM_STATUS_OUT_OF_GAS: &str = "out_of_gas";
pub const VM_STATUS_MOVE_ABORT: &str = "move_abort";
pub const VM_STATUS_EXECUTION_FAILURE: &str = "execution_failure";
pub const VM_STATUS_MISC_ERROR: &str = "miscellaneous_error";

// TransactionData#type field values
pub const TRANSACTION_DATA_BLOCK_METADATA: &str = "blockmetadata";
pub const TRANSACTION_DATA_WRITE_SET: &str = "writeset";
pub const TRANSACTION_DATA_USER: &str = "user";
pub const TRANSACTION_DATA_UNKNOWN: &str = "unknown";

// Script#type field values, only set unknown type here,
// other types, plese see https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/doc/transaction_script_documentation.md for all available script names.
pub const SCRIPT_UNKNOWN: &str = "unknown";
