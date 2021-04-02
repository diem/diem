use crate::{Node, NodeParam, ProcessNodeParam};
use std::process::{Child, Command};
use diem_types::account_address::AccountAddress;
use diem_config::config::{RoleType, NodeConfig};
use debug_interface::NodeDebugClient;
use std::path::PathBuf;
use std::fs::File;
use std::env;
use anyhow::{Context, format_err};

pub struct LocalNode {
    node: Child,
    node_id: String,
    validator_peer_id: Option<AccountAddress>,
    role: RoleType,
    debug_client: NodeDebugClient,
    port: u16,
    pub log: PathBuf,
}

impl Drop for LocalNode {
    // When the DiemNode struct goes out of scope we need to kill the child process
    fn drop(&mut self) {
        // check if the process has already been terminated
        match self.node.try_wait() {
            // The child process has already terminated, perhaps due to a crash
            Ok(Some(_)) => {}

            // The node is still running so we need to attempt to kill it
            _ => {
                if let Err(e) = self.node.kill() {
                    panic!("DiemNode process could not be killed: '{}'", e);
                }
                self.node.wait().unwrap();
            }
        }
    }
}

impl Node for LocalNode {
    fn launch(&self, config: NodeParam) -> anyhow::Result<Box<Self>> {
        match config {
            NodeParam::ProcessNode(param) => {
                let config = NodeConfig::load(&param.config_path)
                    .unwrap_or_else(|_| panic!("Failed to load NodeConfig from file: {:?}", param.config_path));
                let log_file = File::create(&param.log_path)?;
                let validator_peer_id = match param.role {
                    RoleType::Validator => Some(config.validator_network.as_ref().unwrap().peer_id()),
                    RoleType::FullNode => None,
                };
                let mut node_command = Command::new(param.diem_node_bin_path);
                node_command.arg("-f").arg(param.config_path);
                if env::var("RUST_LOG").is_err() {
                    // Only set our RUST_LOG if its not present in environment
                    node_command.env("RUST_LOG", "debug");
                }
                node_command
                    .stdout(log_file.try_clone()?)
                    .stderr(log_file.try_clone()?);
                let node = node_command.spawn().with_context(|| {
                    format!(
                        "Error launching node process with binary: {:?}",
                        param.diem_node_bin_path
                    )
                })?;
                let debug_client = NodeDebugClient::new(
                    "localhost",
                    config.debug_interface.admission_control_node_debug_port,
                );
                Ok(Box::from(Self {
                    node,
                    node_id: param.node_id,
                    validator_peer_id,
                    role: param.role,
                    debug_client,
                    port: config.json_rpc.address.port(),
                    log: param.log_path,
                }))
            }
            _ => return Err(format_err!(
                "Unsupported param type"
                )),
        }
    }
}
