mod commit_check;
mod liveness_check;
mod log_tail;

use crate::cluster::Cluster;
pub use commit_check::CommitHistoryHealthCheck;
use itertools::Itertools;
pub use liveness_check::LivenessHealthCheck;
pub use log_tail::AwsLogTail;
use std::{
    collections::HashMap,
    fmt,
    time::{Duration, SystemTime},
};
use termion::color::*;

#[derive(Clone, Debug)]
pub struct Commit {
    commit: String,
    round: u64,
    parent: String,
}

#[derive(Debug)]
pub enum Event {
    Commit(Commit),
    ConsensusStarted,
}

pub struct ValidatorEvent {
    validator: String,
    timestamp: Duration,
    event: Event,
}

impl fmt::Debug for ValidatorEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {:?}",
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
}

pub struct HealthCheckRunner {
    cluster: Cluster,
    health_checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthCheckRunner {
    pub fn new(cluster: Cluster, health_checks: Vec<Box<dyn HealthCheck>>) -> Self {
        Self {
            cluster,
            health_checks,
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
            for event in events {
                health_check.on_event(event, &mut context);
            }
            health_check.verify(&mut context);
        }
        for err in context.err_acc {
            node_health.insert(err.validator.clone(), false);
            println!("{:?}", err);
        }

        let mut failed = vec![];
        for (i, (node, healthy)) in node_health.into_iter().sorted().enumerate() {
            if healthy {
                print!("{}* {}{}\t", Fg(Green), node, Fg(Reset));
            } else {
                print!("{}* {}{}\t", Fg(Red), node, Fg(Reset));
                failed.push(node);
            }
            if (i + 1) % 5 == 0 {
                println!();
            }
        }
        println!();

        failed
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
