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
    time::Duration,
};

pub struct AwsLogTail {
    pub event_receiver: mpsc::Receiver<ValidatorEvent>,
    #[allow(dead_code)]
    thread: JoinHandle<()>,
}

struct AwsLogThread {
    aws: Aws,
    kinesis_iterator: String,
    event_sender: mpsc::SyncSender<ValidatorEvent>,
}

struct LogEntry<'a> {
    validator: String,
    message: &'a str,
    #[allow(dead_code)]
    timestamp: u64,
}

impl AwsLogTail {
    pub fn spawn_new(aws: Aws) -> failure::Result<Self> {
        let kinesis_iterator = Self::make_kinesis_iterator(&aws)?;
        let (event_sender, event_receiver) = mpsc::sync_channel(10_000);
        let aws_log_thread = AwsLogThread {
            aws,
            kinesis_iterator,
            event_sender,
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
            thread::sleep(Duration::from_millis(100))
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
        let validator = Self::log_stream_to_validator_short_str(log_stream);
        let events = json
            .get("logEvents")
            .expect("No logEvents in kinesis event");
        let events = events
            .as_array()
            .expect("logEvents in kinesis event is not array");
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
            let log_entry = LogEntry {
                validator: validator.clone(),
                message,
                timestamp,
            };
            self.handle_log_entry(log_entry);
        }
        Ok(())
    }

    fn handle_log_entry(&mut self, e: LogEntry) {
        let commit = Self::parse_commit_log_entry(&e);
        if let Some(commit) = commit {
            let _ignore = self.event_sender.send(ValidatorEvent {
                validator: e.validator,
                event: Event::Commit(commit),
            });
        }
    }

    fn log_stream_to_validator_short_str(s: &str) -> String {
        let re = Regex::new(r"^validator-([0-9a-z]+)/").unwrap();
        let cap = re.captures(s).expect("can not parse log stream name");
        cap[1].into()
    }

    fn parse_commit_log_entry(e: &LogEntry) -> Option<Commit> {
        let re = Regex::new(
            r"Committed.+\[id: ([a-z0-9]+), round: ([a-z0-9]+), parent_id: ([a-z0-9]+)]",
        )
        .unwrap();
        let cap = match re.captures(e.message) {
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
}
