// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    json_rpc::JsonRpcClientWrapper,
    validator_set::{validator_set_full_node_addresses, validator_set_validator_addresses},
};
use diem_config::{
    config::{RoleType, HANDSHAKE_VERSION},
    network_id::{NetworkContext, NetworkId},
};
use diem_crypto::{x25519, x25519::PRIVATE_KEY_SIZE};
use diem_management::error::Error;
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::{InMemoryStorage, Storage};
use diem_types::{
    account_address,
    chain_id::ChainId,
    network_address::{
        encrypted::{Key, KeyVersion, KEY_LEN},
        NetworkAddress,
    },
    PeerId,
};
use fallible::copy_from_slice::copy_slice_to_vec;
use netcore::transport::tcp::{resolve_and_connect, TcpSocket};
use network::{
    noise::{HandshakeAuthMode, NoiseUpgrader},
    protocols::wire::handshake::v1::SupportedProtocols,
    transport::{upgrade_outbound, UpgradeContext, SUPPORTED_MESSAGING_PROTOCOL},
    ProtocolId,
};
use std::{collections::BTreeMap, sync::Arc};
use structopt::StructOpt;
use tokio::{runtime::Runtime, time::Duration};

const DEFAULT_TIMEOUT_SECONDS: u64 = 5;

#[derive(Debug, StructOpt)]
pub struct CheckEndpoint {
    /// `NetworkAddress` of remote server interface
    #[structopt(long)]
    address: NetworkAddress,
    /// `ChainId` of remote server
    #[structopt(long)]
    chain_id: ChainId,
    /// `NetworkId` of remote server interface
    #[structopt(long)]
    network_id: NetworkId,
    /// `PrivateKey` to connect to remote server
    #[structopt(long, parse(try_from_str = parse_private_key_hex))]
    private_key: x25519::PrivateKey,
    /// Optional number of seconds to timeout attempting to connect to endpoint
    #[structopt(long)]
    timeout_seconds: Option<u64>,
}

fn parse_private_key_hex(src: &str) -> Result<x25519::PrivateKey, Error> {
    let input = src.trim();
    if input.len() != 64 {
        return Err(Error::CommandArgumentError(
            "Invalid private key length, must be 64 hex characters".to_string(),
        ));
    }

    let value_slice = hex::decode(src.trim())
        .map_err(|_| Error::CommandArgumentError(format!("Not a valid private key: {}", src)))?;

    let mut value = [0; PRIVATE_KEY_SIZE];
    copy_slice_to_vec(&value_slice, &mut value)
        .map_err(|e| Error::CommandArgumentError(format!("{}", e)))?;

    Ok(x25519::PrivateKey::from(value))
}

impl CheckEndpoint {
    pub fn execute(self) -> Result<String, Error> {
        validate_address(&self.address)?;
        let (peer_id, public_key) = private_key_to_public_info(&self.private_key);
        check_endpoint(
            build_upgrade_context(self.chain_id, self.network_id, peer_id, self.private_key),
            self.address,
            peer_id,
            public_key,
            timeout_duration(self.timeout_seconds),
        )
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
    #[structopt(long, required_if("role", "validator"), parse(try_from_str = parse_validator_key_hex))]
    address_encryption_key: Option<Key>,
    /// The expected on-chain key version, only required for validator checks
    #[structopt(long, required_if("role", "validator"))]
    version: Option<KeyVersion>,
    /// `ChainId` of remote server
    #[structopt(long)]
    chain_id: ChainId,
    /// Private key to connect to remote server
    #[structopt(long, parse(try_from_str = parse_private_key_hex))]
    private_key: x25519::PrivateKey,
    /// Optional number of seconds to timeout attempting to connect to endpoint
    #[structopt(long)]
    timeout_seconds: Option<u64>,
}

fn parse_validator_key_hex(src: &str) -> Result<Key, Error> {
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
        let client = JsonRpcClientWrapper::new(self.json_server);

        let nodes = if is_validator {
            // Following unwraps shouldn't fail as it is in memory
            let mut encryptor = Encryptor::new(Storage::InMemoryStorage(InMemoryStorage::new()));
            encryptor.initialize().unwrap();
            encryptor
                .add_key(self.version.unwrap(), self.address_encryption_key.unwrap())
                .unwrap();
            encryptor
                .set_current_version(self.version.unwrap())
                .unwrap();

            validator_set_validator_addresses(client, &encryptor, None)?
        } else {
            validator_set_full_node_addresses(client, None)?
        };

        // Build a single upgrade context to run all the checks
        let network_id = if is_validator {
            NetworkId::Validator
        } else {
            NetworkId::Public
        };

        let (peer_id, public_key) = private_key_to_public_info(&self.private_key);
        let upgrade_context =
            build_upgrade_context(self.chain_id, network_id, peer_id, self.private_key);

        let timeout = timeout_duration(self.timeout_seconds);

        // Check all the addresses accordingly
        for (name, peer_id, addrs) in nodes {
            for addr in addrs {
                match check_endpoint(upgrade_context.clone(), addr, peer_id, public_key, timeout) {
                    Ok(_) => println!("{} -- good", name),
                    Err(err) => println!("{} -- bad -- {}", name, err),
                };
            }
        }

        Ok("Complete!".to_string())
    }
}

/// Builds a listener free noise connector
fn build_upgrade_context(
    chain_id: ChainId,
    network_id: NetworkId,
    peer_id: PeerId,
    private_key: x25519::PrivateKey,
) -> Arc<UpgradeContext> {
    // RoleType doesn't matter, but the `NetworkId` and `PeerId` are used in handshakes
    let network_context = Arc::new(NetworkContext::new(
        RoleType::FullNode,
        network_id.clone(),
        peer_id,
    ));

    // Let's make sure some protocol can be connected.  In the future we may want to allow for specifics
    let mut supported_protocols = BTreeMap::new();
    supported_protocols.insert(
        SUPPORTED_MESSAGING_PROTOCOL,
        SupportedProtocols::from(ProtocolId::all().iter()),
    );

    // Build the noise and network handshake, without running a full Noise server with listener
    Arc::new(UpgradeContext::new(
        NoiseUpgrader::new(
            network_context,
            private_key,
            // If we had an incoming message, auth mode would matter
            HandshakeAuthMode::server_only(),
        ),
        HANDSHAKE_VERSION,
        supported_protocols,
        chain_id,
        network_id,
    ))
}

fn timeout_duration(maybe_secs: Option<u64>) -> Duration {
    Duration::from_secs(if let Some(secs) = maybe_secs {
        secs
    } else {
        DEFAULT_TIMEOUT_SECONDS
    })
}

fn validate_address(address: &NetworkAddress) -> Result<(), Error> {
    if !address.is_diemnet_addr() {
        Err(Error::CommandArgumentError(
            "Address must have ip, tcp, noise key, and handshake".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Wrapper for `check_endpoint_inner` to handle runtime
fn check_endpoint(
    upgrade_context: Arc<UpgradeContext>,
    address: NetworkAddress,
    peer_id: PeerId,
    public_key: x25519::PublicKey,
    timeout: Duration,
) -> Result<String, Error> {
    let runtime = Runtime::new().unwrap();
    let remote_pubkey = address.find_noise_proto().unwrap();
    println!(
        "Connecting with peer_id {} and pubkey {} to {} with timeout: {:?}",
        peer_id, public_key, address, timeout
    );

    let connect_future = async {
        tokio::time::timeout(
            timeout,
            check_endpoint_inner(upgrade_context, address.clone(), remote_pubkey),
        )
        .await
        .map_err(|_| Error::Timeout("CheckEndpoint", address.to_string()))
    };

    runtime.block_on(connect_future)?
}

/// Connects via Noise, and then drops the connection
async fn check_endpoint_inner(
    upgrade_context: Arc<UpgradeContext>,
    address: NetworkAddress,
    remote_pubkey: x25519::PublicKey,
) -> Result<String, Error> {
    // Connect to the address, this should handle DNS resolution
    let fut_socket = async {
        resolve_and_connect(address.clone())
            .await
            .map(TcpSocket::new)
    };

    // The peer id doesn't matter because we don't validate it
    let remote_peer_id = account_address::from_identity_public_key(remote_pubkey);
    match upgrade_outbound(
        upgrade_context,
        fut_socket,
        address.clone(),
        remote_peer_id,
        remote_pubkey,
    )
    .await
    {
        Ok(conn) => {
            let msg = format!("Successfully connected to {}", conn.metadata.addr);

            // Disconnect
            drop(conn);
            Ok(msg)
        }
        Err(error) => Err(Error::UnexpectedError(format!(
            "Failed to connect to {} due to {}",
            address, error
        ))),
    }
}

/// Derive the peer id that we're using.  This is a convenience to only have to provide private key
fn private_key_to_public_info(private_key: &x25519::PrivateKey) -> (PeerId, x25519::PublicKey) {
    let public_key = private_key.public_key();
    let peer_id = account_address::from_identity_public_key(public_key);
    (peer_id, public_key)
}
