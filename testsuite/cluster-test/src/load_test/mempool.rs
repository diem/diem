// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;

use crate::load_test::Handler;
use async_trait::async_trait;
use futures::StreamExt;
use libra_mempool::network::{MempoolNetworkEvents, MempoolNetworkSender};
use network::protocols::network::Event;
use std::fmt;
use std::time::{Duration, Instant};

pub struct Mempool {
    handler: Option<(MempoolNetworkSender, MempoolNetworkEvents)>,
}

impl Mempool {
    pub fn new(handler: Option<(MempoolNetworkSender, MempoolNetworkEvents)>) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl Handler for Mempool {
    async fn start(&mut self, duration: Duration) -> Result<()> {
        let (mempool_sender, mempool_events) = self
            .handler
            .take()
            .expect("missing mempool network handles");
        let mempool_task = Some(tokio::task::spawn(mempool_load_test(
            duration,
            mempool_sender,
            mempool_events,
        )));
        if let Some(t) = mempool_task {
            let _ = t.await.expect("failed mempool load test task");
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        unimplemented!()
    }
}

impl fmt::Display for Mempool {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

async fn mempool_load_test(
    duration: Duration,
    mut sender: MempoolNetworkSender,
    mut events: MempoolNetworkEvents,
) -> Result<MempoolResult> {
    let new_peer_event = events.select_next_some().await?;
    let vfn = if let Event::NewPeer(peer_id, _) = new_peer_event {
        peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for mempool load test"
        ));
    };

    let task_start = Instant::now();
    while Instant::now().duration_since(task_start) < duration {
        let msg = libra_mempool::network::MempoolSyncMsg::BroadcastTransactionsRequest {
            request_id: "0_100".to_string(),
            transactions: vec![], // TODO submit actual txns
        };
        // TODO log stats for bandwidth sent to remote peer to MempoolResult
        sender.send_to(vfn, msg)?;

        // await ACK from remote peer
        let _response = events.select_next_some().await;
    }

    Ok(MempoolResult)
}

// TODO store more stats
struct MempoolResult;
