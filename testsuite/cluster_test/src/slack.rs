use failure::{
    self,
    prelude::{bail, format_err},
};
use reqwest::{self, Url};
use serde_json::{self, json};
use std::env;

pub struct SlackClient {
    url: Url,
    client: reqwest::Client,
}

impl SlackClient {
    pub fn try_new_from_environment() -> Option<Self> {
        match env::var("SLACK_URL") {
            Err(..) => None,
            Ok(url) => {
                let url = url.parse().expect("Failed to parse SLACK_URL");
                let client = reqwest::Client::new();
                Some(Self { url, client })
            }
        }
    }

    pub fn send_message(&self, msg: &str) -> failure::Result<()> {
        let msg = json!({ "text": msg });
        let msg = serde_json::to_string(&msg)
            .map_err(|e| format_err!("Failed to serialize message for slack: {:?}", e))?;
        let request = self.client.post(self.url.clone()).body(msg);
        let response = request
            .send()
            .map_err(|e| format_err!("Failed to send slack message: {:?}", e))?;
        if !response.status().is_success() {
            bail!("Slack service returned error code: {}", response.status())
        }
        Ok(())
    }
}
