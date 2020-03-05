// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    health::{Commit, Event, LogTail, ValidatorEvent},
    instance::Instance,
    util::unix_timestamp_now,
};
use debug_interface::{proto::Event as DebugInterfaceEvent, NodeDebugClient};
use libra_logger::*;
use serde_json::{self, value as json};
use std::{
    env,
    sync::{atomic::AtomicI64, mpsc, Arc},
    thread,
    time::Duration,
};

pub struct DebugPortLogThread {
    instance: Instance,
    client: NodeDebugClient,
    event_sender: mpsc::Sender<ValidatorEvent>,
    started_sender: Option<mpsc::Sender<()>>,
}

impl DebugPortLogThread {
    pub fn spawn_new(cluster: &Cluster) -> LogTail {
        let (event_sender, event_receiver) = mpsc::channel();
        let mut started_receivers = vec![];
        for instance in cluster.validator_instances() {
            let (started_sender, started_receiver) = mpsc::channel();
            started_receivers.push(started_receiver);
            let client = NodeDebugClient::new(instance.ip(), 6191);
            let debug_port_log_thread = DebugPortLogThread {
                instance: instance.clone(),
                client,
                event_sender: event_sender.clone(),
                started_sender: Some(started_sender),
            };
            thread::Builder::new()
                .name(format!("log-tail-{}", instance.peer_name()))
                .spawn(move || debug_port_log_thread.run())
                .expect("Failed to spawn log tail thread");
        }
        for r in started_receivers {
            if let Err(e) = r.recv() {
                panic!("Failed to start one of debug port log threads: {:?}", e);
            }
        }
        LogTail {
            event_receiver,
            pending_messages: Arc::new(AtomicI64::new(0)),
        }
    }
}

impl DebugPortLogThread {
    pub fn run(mut self) {
        let print_failures = env::var("VERBOSE").is_ok();
        loop {
            match self.client.get_events() {
                Err(e) => {
                    if print_failures {
                        info!("Failed to get events from {}: {:?}", self.instance, e);
                    }
                    thread::sleep(Duration::from_secs(1));
                }
                Ok(resp) => {
                    for event in resp.events.into_iter() {
                        if let Some(e) = self.parse_event(event) {
                            let _ignore = self.event_sender.send(e);
                        }
                    }
                    thread::sleep(Duration::from_millis(200));
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
        let json: json::Value =
            serde_json::from_str(&event.json).expect("Failed to parse json from debug interface");

        let e = if event.name == "committed" {
            Self::parse_commit(json)
        } else {
            warn!("Unknown event: {} from {}", event.name, self.instance);
            return None;
        };
        Some(ValidatorEvent {
            validator: self.instance.peer_name().clone(),
            timestamp: Duration::from_millis(event.timestamp as u64),
            received_timestamp: unix_timestamp_now(),
            event: e,
        })
    }

    fn parse_commit(json: json::Value) -> Event {
        Event::Commit(Commit {
            commit: json
                .get("block_id")
                .expect("No block_id in commit event")
                .as_str()
                .expect("block_id is not string")
                .to_string(),
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
