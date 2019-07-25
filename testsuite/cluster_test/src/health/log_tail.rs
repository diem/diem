use crate::{
    aws::Aws,
    health::{Commit, Event, ValidatorEvent},
};
use failure::{self, prelude::*};
use flate2::read::GzDecoder;
use regex::Regex;
use rusoto_kinesis::{GetRecordsInput, GetShardIteratorInput, Kinesis, Record};
use serde_json::{self, value as json};
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};
//use lazy_static::lazy_static;

pub struct AwsLogTail {
    pub event_receiver: mpsc::Receiver<ValidatorEvent>,
    #[allow(dead_code)]
    thread: JoinHandle<()>,
}

struct AwsLogThread {
    aws: Aws,
    kinesis_iterator: String,
    event_sender: mpsc::Sender<ValidatorEvent>,

    re_commit: Regex,
    re_validator: Regex,
    re_started: Regex,
}

struct LogEntry<'a> {
    validator: String,
    message: &'a str,
    timestamp: Duration,
}

impl AwsLogTail {
    pub fn spawn_new(aws: Aws) -> failure::Result<Self> {
        let kinesis_iterator = Self::make_kinesis_iterator(&aws)?;
        let (event_sender, event_receiver) = mpsc::channel();
        let aws_log_thread = AwsLogThread {
            aws,
            kinesis_iterator,
            event_sender,

            re_commit: Regex::new(
                r"Committed.+\[id: ([a-z0-9]+), round: ([a-z0-9]+), parent_id: ([a-z0-9]+)]",
            )
            .unwrap(),
            re_validator: Regex::new(r"^validator-([0-9a-z]+)/").unwrap(),
            re_started: Regex::new(r"Chained BFT SMR started.$").unwrap(),
        };
        let builder = thread::Builder::new();
        let thread = builder
            .name("aws-log-tail".into())
            .spawn(move || aws_log_thread.run())
            .unwrap();
        Ok(AwsLogTail {
            thread,
            event_receiver,
        })
    }

    pub fn recv_all_until_deadline(&self, deadline: Instant) -> Vec<ValidatorEvent> {
        let mut events = vec![];
        while Instant::now() < deadline {
            match self.event_receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(..) => thread::sleep(Duration::from_millis(1)),
            }
        }
        events
    }

    pub fn recv_all(&self) -> Vec<ValidatorEvent> {
        let mut events = vec![];
        while let Ok(event) = self.event_receiver.try_recv() {
            events.push(event);
        }
        events
    }

    fn make_kinesis_iterator(aws: &Aws) -> failure::Result<String> {
        let response = aws
            .kc()
            .get_shard_iterator(GetShardIteratorInput {
                shard_id: "0".into(),
                shard_iterator_type: "LATEST".into(),
                stream_name: format!("{}-RecipientStream", aws.workplace()),
                starting_sequence_number: None,
                timestamp: None,
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
}

impl AwsLogThread {
    fn run(mut self) {
        loop {
            let r = self.aws_poll_iterator().expect("Failed to poll kinesis");
            self.kinesis_iterator = r.0;
            let records = r.1;
            for record in records {
                self.handle_kinesis_record(record)
                    .expect("Failed to process aws record");
            }
            thread::sleep(Duration::from_millis(500))
        }
    }

    fn aws_poll_iterator(&self) -> failure::Result<(String, Vec<Record>)> {
        let response = self
            .aws
            .kc()
            .get_records(GetRecordsInput {
                shard_iterator: self.kinesis_iterator.clone(),
                limit: Some(10000),
            })
            .sync()?;
        let next_iterator = response
            .next_shard_iterator
            .expect("Next iterator is expected for kinesis stream");
        let records = response.records;
        Ok((next_iterator, records))
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
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Now is behind UNIX_EPOCH");
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
        if behind_max > 10_000 {
            println!(
                "Logs behind avg={} ms, max={} ms; {} entries",
                behind_sum / count,
                behind_max,
                count
            );
        }
        Ok(())
    }

    fn handle_log_entry(&mut self, e: LogEntry) {
        let event = self.parse_log_entry(&e);
        if let Some(event) = event {
            let _ignore = self.event_sender.send(ValidatorEvent {
                validator: e.validator,
                timestamp: e.timestamp,
                event,
            });
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
        if let Some(cap) = cap {
            Some(cap[1].into())
        } else {
            None
        }
    }

    fn parse_commit_log_entry(&self, e: &LogEntry) -> Option<Commit> {
        let cap = match self.re_commit.captures(e.message) {
            Some(cap) => cap,
            None => return None,
        };
        let commit = &cap[1];
        let round = &cap[2];
        let parent = &cap[3];
        let round = match round.parse::<u64>() {
            Ok(round) => round,
            Err(..) => return None,
        };
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
