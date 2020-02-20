// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod commit_check;
mod debug_interface_log_tail;
mod liveness_check;
mod log_tail;

use crate::{cluster::Cluster, util::unix_timestamp_now};
use anyhow::{bail, Result};
pub use commit_check::CommitHistoryHealthCheck;
pub use debug_interface_log_tail::DebugPortLogThread;
use itertools::Itertools;
pub use liveness_check::LivenessHealthCheck;
pub use log_tail::LogTail;
use std::{
    collections::HashMap,
    collections::HashSet,
    env, fmt,
    iter::FromIterator,
    time::{Duration, Instant, SystemTime},
};
use termion::color::*;

#[derive(Clone, Debug)]
pub struct Commit {
    commit: String,
    round: u64,
    parent: String,
}

#[derive(Clone, Debug)]
pub enum Event {
    Commit(Commit),
    ConsensusStarted,
}

#[derive(Clone)]
pub struct ValidatorEvent {
    validator: String,
    timestamp: Duration,
    received_timestamp: Duration,
    event: Event,
}

impl fmt::Debug for ValidatorEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "recv: {}; {} {} {:?}",
            self.received_timestamp.as_millis(),
            self.timestamp.as_millis(),
            self.validator,
            self.event
        )
    }
}

pub trait HealthCheck {
    /// Verify specific event
    fn on_event(&mut self, event: &ValidatorEvent, ctx: &mut HealthCheckContext);
    /// Periodic verification (happens even if when no events produced)
    fn verify(&mut self, _ctx: &mut HealthCheckContext) {}
    /// Optionally marks validator as failed, requiring waiting for at least one event from it to
    /// mark it as healthy again
    fn invalidate(&mut self, _validator: &str) {}
    /// Clean is invoked when cluster is wiped
    /// This means that checks like commit history check should wipe internal state
    fn clear(&mut self) {}

    fn name(&self) -> &'static str;
}

pub struct HealthCheckRunner {
    cluster: Cluster,
    health_checks: Vec<Box<dyn HealthCheck>>,
    debug: bool,
}

impl HealthCheckRunner {
    pub fn new(cluster: Cluster, health_checks: Vec<Box<dyn HealthCheck>>) -> Self {
        Self {
            cluster,
            health_checks,
            debug: env::var("HEALTH_CHECK_DEBUG").is_ok(),
        }
    }

    pub fn new_all(cluster: Cluster) -> Self {
        let liveness_health_check = LivenessHealthCheck::new(&cluster);
        Self::new(
            cluster,
            vec![
                Box::new(CommitHistoryHealthCheck::new()),
                Box::new(liveness_health_check),
            ],
        )
    }

    /// Takes a list of affected_validators. If there are validators which failed
    /// which were not part of the experiment, then it returns an Err with a string
    /// of all the unexpected failures.
    /// Otherwise, it returns a list of ALL the failed validators
    /// It also takes print_failures parameter that controls level of verbosity of health check
    pub fn run(
        &mut self,
        events: &[ValidatorEvent],
        affected_validators_set: &HashSet<String>,
        print_failures: PrintFailures,
    ) -> Result<Vec<String>> {
        let mut node_health = HashMap::new();
        for instance in self.cluster.validator_instances() {
            node_health.insert(instance.peer_name().clone(), true);
        }
        let mut messages = vec![];

        let mut context = HealthCheckContext::new();
        for health_check in self.health_checks.iter_mut() {
            let start = Instant::now();
            for event in events {
                health_check.on_event(event, &mut context);
            }
            let events_processed = Instant::now();
            health_check.verify(&mut context);
            let verified = Instant::now();
            if self.debug {
                messages.push(format!(
                    "{} {}, on_event time: {}ms, verify time: {}ms, events: {}",
                    unix_timestamp_now().as_millis(),
                    health_check.name(),
                    (events_processed - start).as_millis(),
                    (verified - events_processed).as_millis(),
                    events.len(),
                ));
            }
        }
        for err in context.err_acc {
            node_health.insert(err.validator.clone(), false);
            messages.push(format!("{} {:?}", unix_timestamp_now().as_millis(), err));
        }

        let mut failed = vec![];
        let mut validators_message = "".to_string();
        for (i, (node, healthy)) in node_health.into_iter().sorted().enumerate() {
            if healthy {
                validators_message.push_str(&format!("{}* {}{}   ", Fg(Green), node, Fg(Reset)));
            } else {
                validators_message.push_str(&format!("{}* {}{}   ", Fg(Red), node, Fg(Reset)));
                failed.push(node);
            }
            if (i + 1) % 15 == 0 {
                validators_message.push_str("\n");
            }
        }
        messages.push(validators_message);
        messages.push(format!(""));
        messages.push(format!(""));

        let affected_validators_set_refs = HashSet::from_iter(affected_validators_set.iter());
        let failed_set: HashSet<&String> = HashSet::from_iter(failed.iter());
        let has_unexpected_failures = !failed_set.is_subset(&affected_validators_set_refs);

        if print_failures.should_print(has_unexpected_failures) {
            messages.iter().for_each(|m| println!("{}", m));
        }

        if has_unexpected_failures {
            let unexpected_failures = failed_set
                .difference(&affected_validators_set_refs)
                .join(",");
            bail!(unexpected_failures);
        }
        Ok(failed)
    }

    pub fn invalidate(&mut self, validator: &str) {
        for hc in self.health_checks.iter_mut() {
            hc.invalidate(validator);
        }
    }

    pub fn clear(&mut self) {
        for hc in self.health_checks.iter_mut() {
            hc.clear();
        }
    }
}

pub enum PrintFailures {
    None,
    UnexpectedOnly,
    All,
}

impl PrintFailures {
    fn should_print(&self, has_unexpected_failures: bool) -> bool {
        match self {
            PrintFailures::None => false,
            PrintFailures::UnexpectedOnly => has_unexpected_failures,
            PrintFailures::All => true,
        }
    }
}

pub struct HealthCheckContext {
    now: Duration,
    err_acc: Vec<HealthCheckError>,
}

#[derive(Debug)]
pub struct HealthCheckError {
    pub validator: String,
    pub message: String,
}

impl HealthCheckContext {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Now is behind UNIX_EPOCH");
        Self {
            now,
            err_acc: vec![],
        }
    }

    pub fn now(&self) -> Duration {
        self.now
    }

    pub fn report_failure(&mut self, validator: String, message: String) {
        self.err_acc.push(HealthCheckError { validator, message })
    }
}

impl Default for HealthCheckContext {
    fn default() -> Self {
        Self::new()
    }
}
