// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    health::{log_tail::TraceTail, Commit, Event, LogTail, ValidatorEvent},
    instance::Instance,
};
use debug_interface::AsyncNodeDebugClient;
use libra_logger::{json_log::JsonLogEntry as DebugInterfaceEvent, *};
use libra_time::duration_since_epoch;
use serde_json::{self, value as json};
use std::{
    env,
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};
use tokio::{runtime::Handle, time};

pub struct DebugPortLogWorker {
    instance: Instance,
    client: AsyncNodeDebugClient,
    event_sender: mpsc::Sender<ValidatorEvent>,
    started_sender: Option<mpsc::Sender<()>>,
    pending_messages: Arc<AtomicI64>,
    trace_sender: mpsc::Sender<(String, DebugInterfaceEvent)>,
    trace_enabled: Arc<AtomicBool>,
}

impl DebugPortLogWorker {
    pub fn spawn_new(cluster: &Cluster) -> (LogTail, TraceTail) {
        let runtime = Handle::current();
        let (event_sender, event_receiver) = mpsc::channel();
        let mut started_receivers = vec![];
        let pending_messages = Arc::new(AtomicI64::new(0));
        let (trace_sender, trace_receiver) = mpsc::channel();
        let trace_enabled = Arc::new(AtomicBool::new(false));
        for instance in cluster.validator_and_fullnode_instances() {
            let (started_sender, started_receiver) = mpsc::channel();
            started_receivers.push(started_receiver);
            let client = instance.debug_interface_client();
            let debug_port_log_worker = DebugPortLogWorker {
                instance: instance.clone(),
                client,
                event_sender: event_sender.clone(),
                started_sender: Some(started_sender),
                pending_messages: pending_messages.clone(),
                trace_sender: trace_sender.clone(),
                trace_enabled: trace_enabled.clone(),
            };
            runtime.spawn(debug_port_log_worker.run());
        }
        for r in started_receivers {
            if let Err(e) = r.recv() {
                panic!("Failed to start one of debug port log threads: {:?}", e);
            }
        }
        (
            LogTail {
                event_receiver,
                pending_messages,
            },
            TraceTail {
                trace_enabled,
                trace_receiver: Mutex::new(trace_receiver),
            },
        )
    }
}

impl DebugPortLogWorker {
    pub async fn run(mut self) {
        let print_failures = env::var("VERBOSE").is_ok();
        loop {
            match self.client.get_events().await {
                Err(e) => {
                    if print_failures {
                        info!("Failed to get events from {}: {:?}", self.instance, e);
                    }
                    time::delay_for(Duration::from_secs(1)).await;
                }
                Ok(resp) => {
                    let mut sent_events = 0i64;
                    for event in resp {
                        if let Some(e) = self.parse_event(event) {
                            let _ignore = self.event_sender.send(e);
                            sent_events += 1;
                        }
                    }
                    self.pending_messages
                        .fetch_add(sent_events, Ordering::Relaxed);
                    time::delay_for(Duration::from_millis(100)).await;
                }
            }
            if let Some(started_sender) = self.started_sender.take() {
                if let Err(e) = started_sender.send(()) {
                    panic!("Failed to send to started_sender: {:?}", e);
                }
            }
        }
    }

    fn parse_event(&self, event: DebugInterfaceEvent) -> Option<ValidatorEvent> {
        let e = if event.name == "committed" {
            Self::parse_commit(&event.json)
        } else {
            if self.trace_enabled.load(Ordering::Relaxed) {
                let peer = self.instance.peer_name().clone();
                let _ignore = self.trace_sender.send((peer, event));
            }
            return None;
        };
        Some(ValidatorEvent {
            validator: self.instance.peer_name().clone(),
            timestamp: Duration::from_millis(event.timestamp as u64),
            received_timestamp: duration_since_epoch(),
            event: e,
        })
    }

    fn parse_commit(json: &json::Value) -> Event {
        Event::Commit(Commit {
            commit: json
                .get("block_id")
                .expect("No block_id in commit event")
                .as_str()
                .expect("block_id is not string")
                .to_string(),
            epoch: json
                .get("epoch")
                .expect("No epoch in commit event")
                .as_u64()
                .expect("epoch is not u64"),
            round: json
                .get("round")
                .expect("No round in commit event")
                .as_u64()
                .expect("round is not u64"),
            parent: json
                .get("parent_id")
                .expect("No parent_id in commit event")
                .as_str()
                .expect("parent_id is not string")
                .to_string(),
        })
    }
}
