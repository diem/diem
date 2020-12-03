// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::GetChunkRequest, chunk_response::GetChunkResponse,
    request_manager::ChunkRequestInfo,
};
use anyhow::Error;
use diem_config::config::PeerNetworkId;
use diem_logger::Schema;
use diem_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, waypoint::Waypoint,
};
use serde::Serialize;

#[derive(Clone, Schema)]
pub struct LogSchema<'a> {
    name: LogEntry,
    event: Option<LogEvent>,
    #[schema(debug)]
    error: Option<&'a Error>,
    #[schema(display)]
    peer: Option<&'a PeerNetworkId>,
    is_upstream_peer: Option<bool>,
    #[schema(display)]
    chunk_request: Option<GetChunkRequest>,
    version: Option<u64>,
    #[schema(display)]
    chunk_response: Option<GetChunkResponse>,
    #[schema(display)]
    waypoint: Option<Waypoint>,
    subscription_name: Option<String>,
    count: Option<usize>,
    reconfig_events: Option<Vec<ContractEvent>>,
    local_li_version: Option<u64>,
    local_synced_version: Option<u64>,
    local_epoch: Option<u64>,
    #[schema(display)]
    ledger_info: Option<LedgerInfoWithSignatures>,
    old_epoch: Option<u64>,
    new_epoch: Option<u64>,
    request_version: Option<u64>,
    target_version: Option<u64>,
    old_multicast_level: Option<usize>,
    new_multicast_level: Option<usize>,
    #[schema(debug)]
    chunk_req_info: Option<&'a ChunkRequestInfo>,
}

impl<'a> LogSchema<'a> {
    pub fn new(name: LogEntry) -> Self {
        Self::new_event(name, None)
    }

    pub fn event_log(name: LogEntry, event: LogEvent) -> Self {
        Self::new_event(name, Some(event))
    }

    fn new_event(name: LogEntry, event: Option<LogEvent>) -> Self {
        Self {
            name,
            event,
            peer: None,
            is_upstream_peer: None,
            error: None,
            chunk_request: None,
            chunk_response: None,
            version: None,
            waypoint: None,
            subscription_name: None,
            reconfig_events: None,
            count: None,
            local_li_version: None,
            local_synced_version: None,
            local_epoch: None,
            ledger_info: None,
            new_epoch: None,
            old_epoch: None,
            request_version: None,
            target_version: None,
            old_multicast_level: None,
            new_multicast_level: None,
            chunk_req_info: None,
        }
    }

    pub fn chunk_req(mut self, request: &GetChunkRequest) -> Self {
        self.chunk_request = Some(request.clone());
        self
    }

    pub fn chunk_resp(mut self, response: &GetChunkResponse) -> Self {
        self.chunk_response = Some(response.clone());
        self
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    Reconfig,
    NewPeer,
    LostPeer,
    Waypoint,
    RuntimeStart,
    ConsensusCommit,
    SyncRequest,
    Timeout,
    LocalState,
    SendChunkRequest,
    ProcessChunkRequest,
    ProcessChunkResponse,
    NetworkError,
    EpochChange,
    CommitFlow,
    Multicast,
    SubscriptionDeliveryFail,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    Initialize,
    CallbackFail,
    Complete,
    Timeout,
    PublishError,
    Fail,

    // SendChunkRequest events
    MissingPeers,
    OldSyncRequest,
    NetworkSendError,
    Success,
    ChunkRequestInfo,

    // ProcessChunkResponse events
    Received,
    SendChunkRequestFail,
    ApplyChunkSuccess,
    ApplyChunkFail,
    PostCommitFail,
    OldResponseLI,

    // ProcessChunkRequest events
    PastEpochRequested,
    DeliverChunk,

    // Multicast network events
    Failover,
    Recover,
}
