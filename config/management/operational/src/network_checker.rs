// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_rpc::JsonRpcClientWrapper;
use diem_config::config::RoleType;
use diem_management::error::Error;
use diem_network_address::{
    encrypted::{Key, KeyVersion, KEY_LEN},
    NetworkAddress,
};
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::{InMemoryStorage, Storage};
use fallible::copy_from_slice::copy_slice_to_vec;
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CheckEndpoint {
    #[structopt(long)]
    address: NetworkAddress,
}

// This will show up as an error in the logs as a bad key 07070707...
// It allows us to ensure that both connections work, and that we can see them in the logs
const INVALID_NOISE_HEADER: &[u8; 152] = &[7; 152];

impl CheckEndpoint {
    pub fn execute(self) -> Result<String, Error> {
        let addrs = self.address.to_socket_addrs().map_err(|err| {
            Error::IO(
                "Failed to resolve address from NetworkAddress".to_string(),
                err,
            )
        })?;

        let mut last_error = std::io::Error::new(std::io::ErrorKind::Other, "");

        // The problem here is the endpoint is not supposed to respond to garbage data.
        // So, we check that we get nothing from the endpoint, and that it resolves & connects with TCP
        for addr in addrs {
            eprintln!("Trying address: {}", addr);
            match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                Ok(mut stream) => {
                    // We should be able to write to the socket dummy data
                    if let Err(error) = stream.write(INVALID_NOISE_HEADER) {
                        eprintln!("Failed to write to address {}", error);
                        last_error = error;
                        continue;
                    }
                    let buf = &mut [0; 1];
                    match stream.read(buf) {
                        Ok(size) => {
                            if size == 0 {
                                // Connection is open, and doesn't return anything
                                // This is the closest we can get to working
                                return Ok(format!(
                                    "Accepted write and responded with nothing at {}",
                                    addr
                                ));
                            } else {
                                eprintln!(
                                    "Endpoint responded with data!  Shouldn't be a noise endpoint."
                                );
                                last_error = std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Responded with data when it shouldn't.",
                                )
                            }
                        }
                        Err(error) => {
                            eprintln!("Failed to read from address {}", error);
                            last_error = error
                        }
                    }
                }
                Err(error) => last_error = error,
            }
        }

        Err(Error::IO(
            "No addresses responded correctly".to_string(),
            last_error,
        ))
    }
}

#[derive(Debug, StructOpt)]
pub struct CheckValidatorSetEndpoints {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: String,
    /// Specifies whether or not to evaluate validators or fullnodes
    #[structopt(long)]
    role: RoleType,
    /// The expected on-chain key, only required for validator checks
    #[structopt(long, required_if("role", "validator"), parse(try_from_str = parse_hex))]
    key: Option<Key>,
    /// The expected on-chain key version, only required for validator checks
    #[structopt(long, required_if("role", "validator"))]
    version: Option<KeyVersion>,
}

fn parse_hex(src: &str) -> Result<Key, Error> {
    let potential_err_msg = format!("Not a valid encryption key: {}", src);
    let value_slice =
        hex::decode(src.trim()).map_err(|_| Error::CommandArgumentError(potential_err_msg))?;

    let mut value = [0; KEY_LEN];
    copy_slice_to_vec(&value_slice, &mut value)
        .map_err(|e| Error::CommandArgumentError(format!("{}", e)))?;
    Ok(value)
}

impl CheckValidatorSetEndpoints {
    pub fn execute(self) -> Result<String, Error> {
        let is_validator = self.role.is_validator();

        // Following unwraps shouldn't fail as it is in memory
        let mut encryptor = Encryptor::new(Storage::InMemoryStorage(InMemoryStorage::new()));
        encryptor.initialize().unwrap();
        let encryptor = if is_validator {
            encryptor
                .add_key(self.version.unwrap(), self.key.unwrap())
                .unwrap();
            encryptor
                .set_current_version(self.version.unwrap())
                .unwrap();
            encryptor
        } else {
            encryptor
        };

        let client = JsonRpcClientWrapper::new(self.json_server);
        let validator_set = crate::validator_set::decode_validator_set(encryptor, client, None)?;

        for info in validator_set {
            let address = if is_validator {
                info.fullnode_network_address.clone()
            } else {
                info.validator_network_address.clone()
            };
            let check_endpoint = CheckEndpoint { address };
            match check_endpoint.execute() {
                Ok(_) => println!("{} -- good", info.name),
                Err(err) => println!("{} -- bad -- {}", info.name, err),
            };
        }

        Ok("Complete!".to_string())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test_helper::OperationalTool;
    use diem_network_address::NetworkAddress;
    use diem_types::chain_id::ChainId;
    use std::str::FromStr;

    #[test]
    fn test_check_endpoint() {
        let op_tool = OperationalTool::new("unused-host".into(), ChainId::test());

        // Check invalid DNS
        let addr = NetworkAddress::from_str("/dns4/diem/tcp/80").unwrap();
        op_tool.check_endpoint(addr).unwrap_err();

        // Check if endpoint responded with data
        let addr = NetworkAddress::from_str("/dns4/diem.org/tcp/80").unwrap();
        op_tool.check_endpoint(addr).unwrap_err();

        // Check bad port
        let addr = NetworkAddress::from_str("/dns4/diem.com/tcp/6180").unwrap();
        op_tool.check_endpoint(addr).unwrap_err();
    }
}
