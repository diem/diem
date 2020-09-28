// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use libra_config::config::PeerNetworkId;
use libra_logger::Schema;
use libra_types::{account_address::AccountAddress, on_chain_config::OnChainConfigPayload};
use serde::Serialize;
use std::fmt;

pub struct TxnsLog {
    txns: Vec<(AccountAddress, u64)>,
    status: Vec<String>,
}

impl TxnsLog {
    pub fn new() -> Self {
        Self {
            txns: vec![],
            status: vec![],
        }
    }

    pub fn new_txn(account: AccountAddress, seq_num: u64) -> Self {
        Self {
            txns: vec![(account, seq_num)],
            status: vec![],
        }
    }

    pub fn add(&mut self, account: AccountAddress, seq_num: u64) {
        self.txns.push((account, seq_num));
    }

    pub fn add_with_status(&mut self, account: AccountAddress, seq_num: u64, status: &str) {
        self.add(account, seq_num);
        self.status.push(status.to_string());
    }
}

impl fmt::Display for TxnsLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut txns = "".to_string();

        for (idx, (account, seq_num)) in self.txns.iter().enumerate() {
            let mut txn = format!("{}:{}", account, seq_num);
            if let Some(status) = self.status.get(idx) {
                txn += &format!(":{}", status)
            }

            txns += &format!("{} ", txn);
        }

        write!(f, "{}", txns)
    }
}

#[derive(Schema)]
pub struct LogSchema<'a> {
    name: LogEntry,
    event: Option<LogEvent>,
    #[schema(debug)]
    error: Option<&'a Error>,
    #[schema(display)]
    peer: Option<&'a PeerNetworkId>,
    is_upstream_peer: Option<bool>,
    #[schema(display)]
    reconfig_update: Option<OnChainConfigPayload>,
    #[schema(display)]
    txns: Option<TxnsLog>,
    account: Option<AccountAddress>,
    //    sequence_number: Option<u64>,
}

impl<'a> LogSchema<'a> {
    pub fn new(name: LogEntry) -> Self {
        Self::new_event(name, None)
    }

    pub fn event_log(name: LogEntry, event: LogEvent) -> Self {
        Self::new_event(name, Some(event))
    }

    pub fn new_event(name: LogEntry, event: Option<LogEvent>) -> Self {
        Self {
            name,
            event,
            error: None,
            peer: None,
            is_upstream_peer: None,
            reconfig_update: None,
            account: None,
            txns: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    NewPeer,
    LostPeer,
    CoordinatorRuntime,
    GCRuntime,
    ReconfigUpdate,
    JsonRpc,
    GetBlock,
    Consensus,
    StateSyncCommit,
    BroadcastTransaction,
    BroadcastACK,

    InvariantViolated,

    AddTxn,
    RemoveTxn,
    MempoolFullEvictedTxn,
    GCRemoveTxns,
    CleanCommittedTxn,
    CleanRejectedTxn,
    ProcessReadyTxns,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    // Runtime events
    Start,
    Live,
    Terminated,

    Received,
    Process,
    VMUpdateFail,

    CallbackFail,
    NetworkSendFail,

    // garbage-collect remove txns events
    SystemTTLExpiration,
    ClientExpiration,

    MissingTxnProcessResult,
}
