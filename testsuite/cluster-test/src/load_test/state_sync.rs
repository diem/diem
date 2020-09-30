// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;

use crate::load_test::Handler;
use async_trait::async_trait;
use futures::StreamExt;
use network::protocols::network::Event;
use state_synchronizer::network::{StateSynchronizerEvents, StateSynchronizerSender};
use std::fmt;
use std::time::{Duration, Instant};

pub struct StateSync {
    handler: Option<(StateSynchronizerSender, StateSynchronizerEvents)>,
}

impl StateSync {
    pub fn new(handler: Option<(StateSynchronizerSender, StateSynchronizerEvents)>) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl Handler for StateSync {
    async fn start(&mut self, duration: Duration) -> Result<()> {
        let (state_sync_sender, state_sync_events) = self
            .handler
            .take()
            .expect("missing state sync network handles");
        let state_sync_task = Some(tokio::task::spawn(state_sync_load_test(
            duration,
            state_sync_sender,
            state_sync_events,
        )));
        if let Some(t) = state_sync_task {
            let _ = t.await.expect("failed state sync load test task");
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

impl fmt::Display for StateSync {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

async fn state_sync_load_test(
    duration: Duration,
    mut sender: StateSynchronizerSender,
    mut events: StateSynchronizerEvents,
) -> Result<StateSyncResult> {
    let new_peer_event = events.select_next_some().await?;
    let vfn = if let Event::NewPeer(peer_id, _) = new_peer_event {
        peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for state sync load test"
        ));
    };

    let chunk_request = state_synchronizer::chunk_request::GetChunkRequest::new(
        1,
        0,
        250,
        state_synchronizer::chunk_request::TargetType::HighestAvailable {
            target_li: None,
            timeout_ms: 10_000,
        },
    );

    let task_start = Instant::now();
    let mut served_txns = 0;
    while Instant::now().duration_since(task_start) < duration {
        let msg = state_synchronizer::network::StateSynchronizerMsg::GetChunkRequest(Box::new(
            chunk_request.clone(),
        ));
        sender.send_to(vfn, msg)?;

        // await response from remote peer
        let response = events.select_next_some().await?;
        if let Event::Message((_remote_peer, payload)) = response {
            if let state_synchronizer::network::StateSynchronizerMsg::GetChunkResponse(
                chunk_response,
            ) = payload
            {
                // TODO analyze response and update StateSyncResult with stats accordingly
                served_txns += chunk_response.txn_list_with_proof.transactions.len();
            }
        }
    }
    Ok(StateSyncResult { served_txns })
}

// TODO store more stats here
struct StateSyncResult {
    pub served_txns: usize,
}
