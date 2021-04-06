use crate::{Node, NodeParam};
use std::process::{Child, Command};
use diem_types::account_address::AccountAddress;
use diem_config::config::{RoleType, NodeConfig};
use std::path::PathBuf;
use std::env;
use anyhow::{format_err, Context};
use std::fs::File;

pub struct ProcessNode {
    node: Child,
    node_id: String,
    validator_peer_id: Option<AccountAddress>,
    role: RoleType,
    debug_interface_port: u32,
    port: u32,
    pub log: PathBuf,
}

impl Drop for ProcessNode {
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

impl Node for ProcessNode {
    fn launch(&self, param: NodeParam) -> anyhow::Result<Box<Self>> {
        match param {
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
                Ok(Box::from(Self {
                    node,
                    node_id: param.node_id,
                    validator_peer_id,
                    role: param.role,
                    debug_interface_port: config.debug_interface.admission_control_node_debug_port as u32,
                    port: config.json_rpc.address.port() as u32,
                    log: param.log_path,
                }))
            }
            _ => return Err(format_err!(
                "Unsupported param type"
                )),
        }
    }

    fn json_rpc_port(&self) -> u32 {
        self.port
    }

    fn debug_interface_port(&self) -> Option<u32> {
        Some(self.debug_interface_port)
    }
}
