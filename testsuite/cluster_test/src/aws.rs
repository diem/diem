use rusoto_core::Region;
use rusoto_ec2::Ec2Client;
use rusoto_kinesis::KinesisClient;

#[derive(Clone)]
pub struct Aws {
    workplace: String,
    kc: KinesisClient,
    ec2: Ec2Client,
}

impl Aws {
    pub fn new(workplace: String) -> Self {
        Self {
            workplace,
            kc: KinesisClient::new(Region::UsWest2),
            ec2: Ec2Client::new(Region::UsWest2),
        }
    }

    pub fn kc(&self) -> &KinesisClient {
        &self.kc
    }

    pub fn ec2(&self) -> &Ec2Client {
        &self.ec2
    }

    pub fn workplace(&self) -> &String {
        &self.workplace
    }
}
