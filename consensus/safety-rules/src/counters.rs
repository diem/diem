use crate::simple_push_metrics::{Counter, Metrics, SerCounter};
use std::sync::atomic::{AtomicI64, Ordering};

pub struct SafetyRulesMetrics {
    pub num_requests_received: Counter,
    pub num_responses_sent: Counter,
}

impl SafetyRulesMetrics {
    pub fn new() -> Self {
        Self {
            num_requests_received: Counter {
                counter_name: "num_requests_received".to_string(),
                counter: AtomicI64::new(0),
            },
            num_responses_sent: Counter {
                counter_name: "num_responses_sent".to_string(),
                counter: AtomicI64::new(0),
            },
        }
    }
}

impl Metrics for SafetyRulesMetrics {
    fn get_metrics(&self) -> Vec<&Counter> {
        vec![&self.num_requests_received, &self.num_responses_sent]
    }
}
