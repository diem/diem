use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex,
    },
    thread,
    thread::JoinHandle,
    time::Duration,
};

pub struct Counter {
    pub counter_name: String,
    pub counter: AtomicI64,
}

pub struct SerCounter {
    pub counter_name: String,
    pub counter: i64,
}

pub trait Metrics {
    fn get_metrics(&self) -> Vec<&Counter>;
}

pub struct CountersPusher {}

impl CountersPusher {
    pub fn start(&self, metrics: Arc<dyn Metrics + Send + Sync>) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let mut data = "".to_string();
            for metric in metrics.get_metrics() {
                let metric_data = format!(
                    "{} {}\n",
                    metric.counter_name,
                    metric.counter.load(Ordering::SeqCst)
                );
                data.push_str(&metric_data);
            }
            let response = ureq::post("http://pushgateway.example.org:9091/metrics/job/safety_rules")
              .timeout_connect(10_000)
              .send_string(&data);
            if response.ok() {
                println!("Ok")
            } else {
                println!("Error")
            }
            thread::sleep(Duration::from_secs(10));
        })
    }
}
