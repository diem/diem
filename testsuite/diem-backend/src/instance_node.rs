use reqwest::{Client, Url};
use std::fmt;
use crate::{Node, NodeParam};
use anyhow::format_err;
use std::str::FromStr;
use diem_client::Client as JsonRpcClient;

#[derive(Clone)]
pub struct InstanceNode {
    peer_name: String,
    ip: String,
    json_rpc_port: u32,
    debug_interface_port: Option<u32>,
    http_client: Client,
}

impl Node for InstanceNode {
    fn launch(&self, config: NodeParam) -> anyhow::Result<Box<Self>> {
        match config {
            NodeParam::InstanceNode(param) => {
                Ok(Box::from(Self {
                    peer_name: param.peer_name,
                    ip: param.ip,
                    json_rpc_port: param.json_rpc_port,
                    debug_interface_port: param.debug_interface_port,
                    http_client: param.http_client
                }))
            }
            _ => return Err(format_err!(
                "Unsupported param type"
                )),
        }
    }

    fn json_rpc_port(&self) -> u32 {
        self.json_rpc_port
    }

    fn debug_interface_port(&self) -> Option<u32> {
        self.debug_interface_port
    }
}

impl InstanceNode {
    pub fn peer_name(&self) -> &str {
        &self.peer_name
    }

    pub fn http_client(&self) -> Client {
        self.http_client.clone()
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn json_rpc_url(&self) -> Url {
        Url::from_str(&format!("http://{}:{}/v1", self.ip(), self.json_rpc_port())).expect("Invalid URL.")
    }

    pub fn json_rpc_client(&self) -> JsonRpcClient {
        JsonRpcClient::new(self.json_rpc_url().to_string())
    }
}

impl fmt::Display for InstanceNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.peer_name, self.ip)
    }
}

impl fmt::Debug for InstanceNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}