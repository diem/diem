use crate::{aws::Aws, util::unix_timestamp_now};
use rusoto_core::Region;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DeleteLogStreamRequest, DescribeLogStreamsRequest,
};
use std::{thread, time::Duration};

pub struct LogPruner {
    aws: Aws,
    aws_logs: CloudWatchLogsClient,
}

impl LogPruner {
    pub fn new(aws: Aws) -> Self {
        let aws_logs = CloudWatchLogsClient::new(Region::UsWest2);
        Self { aws, aws_logs }
    }

    pub fn prune_logs(&self) {
        let log_group = self.aws.workplace();
        let mut next_token = None;
        let now = unix_timestamp_now();
        let mut errors = 0usize;
        let mut deleted = 0usize;
        let max_age = Duration::from_secs(3 * 24 * 3600); //3 days
        loop {
            let mut request = DescribeLogStreamsRequest::default();
            request.limit = Some(50);
            request.log_group_name = log_group.to_string();
            request.order_by = Some("LastEventTime".to_string());
            request.next_token = next_token.clone();
            let response = self.aws_logs.describe_log_streams(request).sync();
            let response = match response {
                Err(err) => {
                    errors += 1;
                    if errors > 5 {
                        panic!("Too many aws errors, aborting");
                    }
                    println!("Retrying aws error: {:?}", err);
                    thread::sleep(Duration::from_millis(600));
                    continue;
                }
                Ok(r) => r,
            };
            errors = 0;
            let streams = response.log_streams.expect("No log_streams");
            for stream in streams {
                let last_event = stream.last_event_timestamp.unwrap_or(0);
                let last_event = Duration::from_millis(last_event as u64);
                if now > last_event {
                    let age = now - last_event;
                    if age < max_age {
                        continue;
                    }
                    let log_stream_name = stream.log_stream_name.expect("No log_stream_name");
                    loop {
                        let request = DeleteLogStreamRequest {
                            log_group_name: log_group.clone(),
                            log_stream_name: log_stream_name.clone(),
                        };
                        let response = self.aws_logs.delete_log_stream(request).sync();
                        if let Err(e) = response {
                            println!("Retrying aws error during log stream removal: {:?}", e);
                            thread::sleep(Duration::from_millis(600));
                            continue;
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(200));
                    deleted += 1;
                    if deleted % 100 == 0 {
                        println!("Deleted {} streams", deleted);
                    }
                }
            }
            next_token = response.next_token;
            if next_token.is_none() {
                break;
            }
            // AWS limit is 5 calls to describe_log_streams per second
            thread::sleep(Duration::from_millis(200));
        }
    }
}
