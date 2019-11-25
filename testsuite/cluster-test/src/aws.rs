// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use rusoto_core::Region;
use rusoto_ec2::Ec2Client;
use rusoto_ecr::EcrClient;
use rusoto_ecs::EcsClient;

#[derive(Clone)]
pub struct Aws {
    workplace: String,
    ec2: Ec2Client,
    ecr: EcrClient,
    ecs: EcsClient,
}

impl Aws {
    pub fn new(workplace: String) -> Self {
        Self {
            workplace,
            ec2: Ec2Client::new(Region::UsWest2),
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

    pub fn workplace(&self) -> &String {
        &self.workplace
    }

    pub fn region(&self) -> &str {
        Region::UsWest2.name()
    }
}
