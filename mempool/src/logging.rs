// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared_mempool::{
    peer_manager::BatchId,
    types::{CommitNotification, ConsensusRequest},
};
use anyhow::Error;
use diem_config::{config::PeerNetworkId, network_id::NetworkId};
use diem_logger::Schema;
use diem_types::{account_address::AccountAddress, on_chain_config::OnChainConfigPayload};
use serde::Serialize;
use std::{fmt, time::SystemTime};

pub struct TxnsLog {
    txns: Vec<(AccountAddress, u64, Option<String>, Option<SystemTime>)>,
}

impl TxnsLog {
    pub fn new() -> Self {
        Self { txns: vec![] }
    }

    pub fn new_txn(account: AccountAddress, seq_num: u64) -> Self {
        Self {
            txns: vec![(account, seq_num, None, None)],
        }
    }

    pub fn add(&mut self, account: AccountAddress, seq_num: u64) {
        self.txns.push((account, seq_num, None, None));
    }

    pub fn add_with_status(&mut self, account: AccountAddress, seq_num: u64, status: &str) {
        self.txns
            .push((account, seq_num, Some(status.to_string()), None));
    }

    pub fn add_full_metadata(
        &mut self,
        account: AccountAddress,
        seq_num: u64,
        status: &str,
        timestamp: Option<SystemTime>,
    ) {
        self.txns
            .push((account, seq_num, Some(status.to_string()), timestamp));
    }
}

impl fmt::Display for TxnsLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut txns = "".to_string();

        for (account, seq_num, status, timestamp) in self.txns.iter() {
            let mut txn = format!("{}:{}", account, seq_num);
            if let Some(status) = status {
                txn += &format!(":{}", status)
            }
            if let Some(timestamp) = timestamp {
                txn += &format!(":{:?}", timestamp)
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
    #[schema(display)]
    consensus_msg: Option<&'a ConsensusRequest>,
    #[schema(display)]
    state_sync_msg: Option<&'a CommitNotification>,
    network_level: Option<u64>,
    upstream_network: Option<&'a NetworkId>,
    #[schema(debug)]
    batch_id: Option<&'a BatchId>,
    backpressure: Option<bool>,
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
            consensus_msg: None,
            state_sync_msg: None,
            network_level: None,
            upstream_network: None,
            batch_id: None,
            backpressure: None,
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
    ReceiveACK,
    InvariantViolated,
    AddTxn,
    RemoveTxn,
    MempoolFullEvictedTxn,
    GCRemoveTxns,
    CleanCommittedTxn,
    CleanRejectedTxn,
    ProcessReadyTxns,
    DBError,
    UpstreamNetwork,
    UnexpectedNetworkMsg,
    MempoolSnapshot,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    // Runtime events
    Start,
    Live,
    Terminated,

    // VM reconfig events
    Received,
    Process,
    VMUpdateFail,

    CallbackFail,
    NetworkSendFail,

    // garbage-collect txns events
    SystemTTLExpiration,
    ClientExpiration,

    Success,
}
