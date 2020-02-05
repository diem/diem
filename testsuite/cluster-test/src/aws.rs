// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{bail, format_err, Result};
use reqwest::{self, Url};
use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use rusoto_ecr::EcrClient;
use rusoto_ecs::EcsClient;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use slog_scope::*;
use std::fs::File;
use std::io::Read;
use std::{thread, time::Duration};

#[derive(Clone)]
pub struct Aws {
    workspace: String,
    ec2: Ec2Client,
    ecr: EcrClient,
    ecs: EcsClient,
}

impl Aws {
    pub fn new() -> Self {
        let ec2 = Ec2Client::new(Region::UsWest2);
        let workspace = discover_workspace(&ec2);
        Self {
            workspace,
            ec2,
            ecr: EcrClient::new(Region::UsWest2),
            ecs: EcsClient::new(Region::UsWest2),
        }
    }

    pub fn ec2(&self) -> &Ec2Client {
        &self.ec2
    }

    pub fn ecr(&self) -> &EcrClient {
        &self.ecr
    }

    pub fn ecs(&self) -> &EcsClient {
        &self.ecs
    }

    pub fn workspace(&self) -> &String {
        &self.workspace
    }

    pub fn region(&self) -> &str {
        Region::UsWest2.name()
    }
}

fn discover_workspace(ec2: &Ec2Client) -> String {
    let instance_id = current_instance_id();
    let mut attempt = 0;
    loop {
        let result = match ec2
            .describe_instances(DescribeInstancesRequest {
                filters: None,
                max_results: None,
                dry_run: None,
                instance_ids: Some(vec![instance_id.clone()]),
                next_token: None,
            })
            .sync()
        {
            Ok(result) => result,
            Err(e) => {
                attempt += 1;
                if attempt > 10 {
                    panic!("Failed to discover workspace");
                }
                error!(
                    "Transient failure when discovering workspace(attempt {}): {}",
                    attempt, e
                );
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        let reservation = result
            .reservations
            .expect("discover_workspace: no reservations")
            .remove(0)
            .instances
            .expect("discover_workspace: no instances")
            .remove(0);
        let tags = reservation.tags.expect("discover_workspace: no tags");
        for tag in tags.iter() {
            if tag.key == Some("Workspace".to_string()) {
                return tag
                    .value
                    .as_ref()
                    .expect("discover_workspace: no tag value")
                    .to_string();
            }
        }
        panic!(
            "discover_workspace: no workspace tag. Instance id: {}, tags: {:?}",
            instance_id, tags
        );
    }
}

fn current_instance_id() -> String {
    let client = reqwest::blocking::Client::new();
    let url = Url::parse("http://169.254.169.254/1.0/meta-data/instance-id");
    let url = url.expect("Failed to parse metadata url");
    let response = client.get(url).send();
    let response = response.expect("Metadata request failed");
    response.text().expect("Failed to parse metadata response")
}

pub fn upload_to_s3(local_filename: &str, bucket: &str, dest_filename: &str) -> Result<()> {
    let mut f = File::open(local_filename).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    match f.read_to_end(&mut contents) {
        Err(e) => bail!("Error opening file to send to S3: {}", e),
        Ok(_) => {
            let req = PutObjectRequest {
                bucket: bucket.to_owned(),
                key: dest_filename.to_owned(),
                body: Some(contents.into()),
                ..Default::default()
            };
            S3Client::new(Region::UsWest2)
                .put_object(req)
                .sync()
                .map_err(|e| format_err!("Failed to upload to S3: {:?}", e))
                .map(|_| ())
        }
    }
}
