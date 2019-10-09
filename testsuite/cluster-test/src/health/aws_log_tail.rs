use crate::{
    aws::Aws,
    cluster::Cluster,
    health::{Commit, Event, LogTail, ValidatorEvent},
    util::unix_timestamp_now,
};
use failure::{self, prelude::*};
use flate2::read::GzDecoder;
use regex::Regex;
use rusoto_kinesis::{GetRecordsInput, GetShardIteratorInput, Kinesis, Record};
use serde_json::{self, value as json};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Write,
    sync::{
        atomic::{AtomicI64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub struct AwsLogThread {
    aws: Aws,
    kinesis_iterator: String,
    event_sender: mpsc::Sender<ValidatorEvent>,
    startup_sender: Option<mpsc::Sender<()>>,
    event_log_file: Option<File>,
    last_seen_ts_by_validator: HashMap<String, Duration>,
    expected_validators_count: usize,
    pending_messages: Arc<AtomicI64>,

    re_commit: Regex,
    re_validator: Regex,
    re_started: Regex,
}

struct LogEntry<'a> {
    validator: String,
    message: &'a str,
    timestamp: Duration,
}

impl AwsLogThread {
    pub fn spawn_new(aws: Aws, cluster: &Cluster) -> failure::Result<LogTail> {
        let kinesis_iterator = Self::make_kinesis_iterator(&aws)?;
        let (event_sender, event_receiver) = mpsc::channel();
        // We block this method until first request to kinesis completed
        let (startup_sender, startup_receiver) = mpsc::channel();
        let event_log_file = match env::var("EVENT_LOG") {
            Ok(f) => Some(File::create(f).expect("Can't create EVENT_LOG file")),
            Err(..) => None,
        };
        let pending_messages = Arc::new(AtomicI64::new(0));
        let aws_log_thread = AwsLogThread {
            aws,
            kinesis_iterator,
            event_sender,
            startup_sender: Some(startup_sender),
            event_log_file,
            last_seen_ts_by_validator: HashMap::new(),
            expected_validators_count: cluster.instances().len(),
            pending_messages: pending_messages.clone(),

            re_commit: Regex::new(
                r"Committed.+\[id: ([a-z0-9]+), round: ([a-z0-9]+), parent_id: ([a-z0-9]+)]",
            )
            .unwrap(),
            re_validator: Regex::new(r"^validator-([0-9a-z]+)/").unwrap(),
            re_started: Regex::new(r"Chained BFT SMR started.$").unwrap(),
        };
        let builder = thread::Builder::new();
        builder
            .name("aws-log-tail".into())
            .spawn(move || aws_log_thread.run())
            .unwrap();
        startup_receiver
            .recv()
            .expect("Aws log tail thread died after first request");
        Ok(LogTail {
            event_receiver,
            pending_messages: pending_messages.clone(),
        })
    }

    fn log_start_offset_sec() -> Option<u64> {
        match env::var("LOG_START_OFFSET") {
            Ok(s) => Some(s.parse().expect("LOG_START_OFFSET env is not a number")),
            Err(..) => None,
        }
    }

    fn make_kinesis_iterator(aws: &Aws) -> failure::Result<String> {
        let stream_name = match env::var("KINESIS_STREAM") {
            Err(..) => format!("JsonEvents-{}-{}", aws.region(), aws.workplace()),
            Ok(s) => s,
        };
        let (shard_iterator_type, timestamp) = match Self::log_start_offset_sec() {
            Some(offset) => {
                let timestamp = unix_timestamp_now() - Duration::from_secs(offset);
                ("AT_TIMESTAMP", Some(timestamp.as_secs() as f64))
            }
            None => ("LATEST", None),
        };
        let response = aws
            .kc()
            .get_shard_iterator(GetShardIteratorInput {
                shard_id: "0".into(),
                shard_iterator_type: shard_iterator_type.into(),
                stream_name,
                starting_sequence_number: None,
                timestamp,
            })
            .sync()?;
        if let Some(shard_iterator) = response.shard_iterator {
            Ok(shard_iterator)
        } else {
            Err(format_err!(
                "Can not make kinesis iterator - empty response"
            ))
        }
    }

    fn run(mut self) {
        let startup_timeout_sec = match env::var("STARTUP_TIMEOUT") {
            Err(..) => 50u64,
            Ok(v) => v.parse().expect("Failed to parse STARTUP_TIMEOUT env"),
        };
        let startup_deadline = Instant::now() + Duration::from_secs(startup_timeout_sec);
        let mut kinesis_fail_retries = 0usize;
        loop {
            let response = self
                .aws
                .kc()
                .get_records(GetRecordsInput {
                    shard_iterator: self.kinesis_iterator.clone(),
                    limit: Some(10000),
                })
                .sync();
            let response = match response {
                Err(e) => {
                    println!("Kinesis failure: {:?}, retry {}", e, kinesis_fail_retries);
                    kinesis_fail_retries += 1;
                    if kinesis_fail_retries > 10 {
                        panic!("Too many kinesis failures");
                    }
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
                Ok(r) => r,
            };
            kinesis_fail_retries = 0;
            let next_iterator = response
                .next_shard_iterator
                .expect("Next iterator is expected for kinesis stream");
            let records = response.records;
            let millis_behind = response
                .millis_behind_latest
                .expect("no millis_behind_latest in kinesis response");
            self.kinesis_iterator = next_iterator;
            self.write_event_log(format!(
                "Kinesis response {} records, millis behind: {}",
                records.len(),
                millis_behind,
            ));
            for record in records {
                self.handle_kinesis_record(record)
                    .expect("Failed to process aws record");
            }
            if self.startup_sender.is_some() && millis_behind == 0 {
                if self.last_seen_ts_by_validator.len() >= self.expected_validators_count {
                    println!("Received events from all validators");
                    self.startup_sender
                        .take()
                        .unwrap()
                        .send(())
                        .expect("Startup receiver dropped");
                } else if Instant::now() > startup_deadline {
                    println!("Aws log startup deadline reached");
                    self.startup_sender
                        .take()
                        .unwrap()
                        .send(())
                        .expect("Startup receiver dropped");
                }
            }
            thread::sleep(Duration::from_millis(300))
        }
    }

    fn handle_kinesis_record(&mut self, record: Record) -> failure::Result<()> {
        let decoder = GzDecoder::new(&record.data[..]);
        let json: json::Value = serde_json::from_reader(decoder)?;
        let log_stream = json
            .get("logStream")
            .expect("No logStream in kinesis event")
            .as_str()
            .expect("logStream in kinesis event is not a string");
        let validator = self.log_stream_to_validator_short_str(log_stream);
        let validator = if let Some(validator) = validator {
            validator
        } else {
            return Ok(());
        };
        let events = json
            .get("logEvents")
            .expect("No logEvents in kinesis event");
        let events = events
            .as_array()
            .expect("logEvents in kinesis event is not array");
        self.write_event_log(format!(
            "Kinesis record for {}, {} events",
            validator,
            events.len()
        ));
        let now = unix_timestamp_now();
        let mut behind_max = 0u128;
        let mut behind_sum = 0u128;
        let mut count = 0u128;
        for event in events {
            let message = event.get("message").expect("No message in kinesis event");
            let message = message
                .as_str()
                .expect("message in kinesis event is not a string");
            let timestamp = event
                .get("timestamp")
                .expect("No timestamp in kinesis event");
            let timestamp = timestamp
                .as_u64()
                .expect("timestamp in kinesis event is not a u64");
            let timestamp = Duration::from_millis(timestamp);
            if now > timestamp {
                let behind_millis = (now - timestamp).as_millis();
                if behind_millis > behind_max {
                    behind_max = behind_millis;
                }
                behind_sum += behind_millis;
            }
            count += 1;
            let log_entry = LogEntry {
                validator: validator.clone(),
                message,
                timestamp,
            };
            self.handle_log_entry(log_entry);
        }
        if behind_max > 15_000 {
            println!(
                "Logs behind avg={} ms, max={} ms; {} entries",
                behind_sum / count,
                behind_max,
                count
            );
        }
        Ok(())
    }

    fn write_event_log(&mut self, s: String) {
        let event_log_file = if let Some(ref mut event_log_file) = self.event_log_file {
            event_log_file
        } else {
            return;
        };
        let now = unix_timestamp_now().as_millis();
        writeln!(event_log_file, "{} {}", now, s).expect("Can't write to EVENT_LOG file");
        event_log_file.flush().expect("Can't flush EVENT_LOG file");
    }

    fn handle_log_entry(&mut self, e: LogEntry) {
        let received_timestamp = unix_timestamp_now();
        let event = self.parse_log_entry(&e);
        if let Some(event) = event {
            let ve = ValidatorEvent {
                received_timestamp,
                validator: e.validator,
                timestamp: e.timestamp,
                event,
            };
            // In rare cases Kinesis delivers messages out of order
            // We are ignoring them so that invariant in CommitHistoryHealthCheck does not break
            let skip = if let Some(last) = self.last_seen_ts_by_validator.get(&ve.validator) {
                *last > ve.timestamp
            } else {
                false
            };
            let skip_msg = if skip { "; [OUT_OF_ORDER]" } else { "" };
            self.write_event_log(format!("{:?}{}", ve, skip_msg));
            if !skip {
                self.last_seen_ts_by_validator
                    .insert(ve.validator.clone(), ve.timestamp);
                self.pending_messages.fetch_add(1, Ordering::Relaxed);
                let _ignore = self.event_sender.send(ve);
            }
        }
    }

    fn parse_log_entry(&self, e: &LogEntry) -> Option<Event> {
        let commit = self.parse_commit_log_entry(&e);
        if let Some(commit) = commit {
            return Some(Event::Commit(commit));
        }
        let consensus_started = self.parse_consensus_started_log_entry(&e);
        if consensus_started.is_some() {
            return Some(Event::ConsensusStarted);
        }
        None
    }

    fn log_stream_to_validator_short_str(&self, s: &str) -> Option<String> {
        let cap = self.re_validator.captures(s);
        cap.map(|cap| cap[1].into())
    }

    fn parse_commit_log_entry(&self, e: &LogEntry) -> Option<Commit> {
        let cap = self.re_commit.captures(e.message)?;
        let commit = &cap[1];
        let round = &cap[2];
        let parent = &cap[3];
        let round = round.parse::<u64>().ok()?;
        Some(Commit {
            commit: commit.into(),
            round,
            parent: parent.into(),
        })
    }

    fn parse_consensus_started_log_entry(&self, e: &LogEntry) -> Option<()> {
        if self.re_started.is_match(e.message) {
            Some(())
        } else {
            None
        }
    }
}
