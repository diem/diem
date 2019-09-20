mod aws_log_tail;
mod commit_check;
mod debug_interface_log_tail;
mod liveness_check;
mod log_tail;

use crate::{cluster::Cluster, util::unix_timestamp_now};
pub use aws_log_tail::AwsLogThread;
pub use commit_check::CommitHistoryHealthCheck;
pub use debug_interface_log_tail::DebugPortLogThread;
use itertools::Itertools;
pub use liveness_check::LivenessHealthCheck;
pub use log_tail::LogTail;
use std::{
    collections::HashMap,
    env, fmt,
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

    /// Returns list of failed validators
    pub fn run(&mut self, events: &[ValidatorEvent]) -> Vec<String> {
        let mut node_health = HashMap::new();
        for instance in self.cluster.instances() {
            node_health.insert(instance.short_hash().clone(), true);
        }

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
                println!(
                    "{} {}, on_event time: {}ms, verify time: {}ms, events: {}",
                    unix_timestamp_now().as_millis(),
                    health_check.name(),
                    (events_processed - start).as_millis(),
                    (verified - events_processed).as_millis(),
                    events.len(),
                );
            }
        }
        for err in context.err_acc {
            node_health.insert(err.validator.clone(), false);
            println!("{} {:?}", unix_timestamp_now().as_millis(), err);
        }

        let mut failed = vec![];
        for (i, (node, healthy)) in node_health.into_iter().sorted().enumerate() {
            if healthy {
                print!("{}* {}{}   ", Fg(Green), node, Fg(Reset));
            } else {
                print!("{}* {}{}   ", Fg(Red), node, Fg(Reset));
                failed.push(node);
            }
            if (i + 1) % 15 == 0 {
                println!();
            }
        }
        println!();
        println!();

        failed
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
