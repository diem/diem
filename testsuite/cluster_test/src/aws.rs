use rusoto_core::Region;
use rusoto_kinesis::KinesisClient;

#[derive(Clone)]
pub struct Aws {
    workplace: String,
    kc: KinesisClient,
}

impl Aws {
    pub fn new(workplace: String) -> Self {
        Self {
            workplace,
            kc: KinesisClient::new(Region::UsWest2),
        }
    }

    pub fn kc(&self) -> &KinesisClient {
        &self.kc
    }

    pub fn workplace(&self) -> &String {
        &self.workplace
    }
}
