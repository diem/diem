// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Client;
use once_cell::sync::Lazy;
use std::{
    net::{TcpListener, TcpStream},
    process::{Child, Command, Stdio},
};

static DIEM_VAULT: Lazy<Option<VaultRunner>> = Lazy::new(|| match VaultRunner::run() {
    Err(err) => {
        assert!(
            std::env::var("DIEM_REQUIRE_VAULT_TESTS").is_err(),
            "Vault is not running: {}",
            err
        );
        println!("Vault is not running: {}", err);
        None
    }
    Ok(vr) => Some(vr),
});

/// This will return the vault host, if vault was started successfully. If vault is expected to be
/// available, an assertion will cause this to fail.
pub fn test_host_safe() -> Option<String> {
    DIEM_VAULT.as_ref().map(|v| v.host().to_string())
}

/// This will return the vault host or panic.
pub fn test_host() -> String {
    test_host_safe().unwrap()
}

const BINARY: &str = "vault";
const HOST: &str = "http://localhost";
pub const ROOT_TOKEN: &str = "root_token";

/// Provide an instance of Vault if vault is installed on the current machine and within the
/// default path.
pub struct VaultRunner {
    _child: Child,
    host: String,
}

impl VaultRunner {
    /// Instantiates a new instance of Vault if one is available or returns a String error stating
    /// where it was unable to make progress.
    pub fn run() -> Result<Self, String> {
        let port = Self::_port()?;

        let mut vault_run = Command::new(BINARY);
        vault_run
            .arg("server")
            .arg("-dev")
            .arg(format!("-dev-root-token-id={}", ROOT_TOKEN).as_str())
            .arg(format!("-dev-listen-address=127.0.0.1:{}", port).as_str());

        let child = vault_run
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;

        let host = format!("{}:{}", HOST, port);
        Self::_transit(&host)?;

        Ok(Self {
            _child: child,
            host,
        })
    }

    // Acquire a random, ephemeral port for Vault
    fn _port() -> Result<u16, String> {
        let listener = TcpListener::bind(("localhost", 0)).map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;

        // Create and accept a connection (which we'll promptly drop) in order to force the port
        // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
        // amount of time (roughly 60s on some Linux systems)
        let _sender = TcpStream::connect(addr).map_err(|e| e.to_string())?;
        let _incoming = listener.accept().map_err(|e| e.to_string())?;
        Ok(addr.port())
    }

    // Turn on transit. This runs a few times to give vault a chance to fully initialize.
    fn _transit(host: &str) -> Result<(), String> {
        let mut count = 0;
        loop {
            let mut transit = Command::new(BINARY);
            let result = transit
                .arg("secrets")
                .arg("enable")
                .arg("-tls-skip-verify")
                .arg(format!("-address={}", host).as_str())
                .arg("transit")
                .output();
            if let Ok(output) = &result {
                if output.status.success() {
                    return Ok(());
                }
            }
            if count == 5 {
                return Err(format!("{:?}", result));
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            count += 1;
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn root_token(&self) -> &str {
        ROOT_TOKEN
    }

    pub fn client(&self) -> Client {
        Client::new(
            self.host().to_string(),
            self.root_token().to_string(),
            None,
            None,
            None,
        )
    }
}

#[test]
fn run_vault() {
    let vr = VaultRunner::run();
    if let Err(err) = vr {
        assert!(
            std::env::var("DIEM_REQUIRE_VAULT_TESTS").is_err(),
            "Vault is not running: {}",
            err
        );
        println!("Vault is not running: {}", err);
    } else if let Ok(vr) = vr {
        vr.client().unsealed().unwrap();
    }
}

#[test]
fn run_test_vault() {
    if let Some(host) = test_host_safe() {
        Client::new(host, ROOT_TOKEN.to_string(), None, None, None)
            .unsealed()
            .unwrap();
    }
}
