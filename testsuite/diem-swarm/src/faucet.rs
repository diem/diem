// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Context, Result};
use diem_types::chain_id::ChainId;
use reqwest::{blocking, StatusCode, Url};
use std::{
    path::Path,
    process::{Child, Command, Stdio},
};

pub struct Process {
    port: u16,
    process: Child,
}

impl Drop for Process {
    fn drop(&mut self) {
        // Kill process process if still running.
        match self.process.try_wait().unwrap() {
            Some(status) => {
                if !status.success() {
                    panic!(
                        "Process terminated with status: {}",
                        status.code().unwrap_or(-1)
                    );
                }
            }
            None => {
                self.process.kill().unwrap();
            }
        }
    }
}

impl Process {
    pub fn start(
        faucet_bin_path: &Path,
        port: u16,
        server_port: u16,
        diem_root_key_path: &Path,
    ) -> Self {
        Self {
            port,
            process: Command::new(faucet_bin_path)
                .arg("-s")
                .arg(format!("http://localhost:{}", server_port))
                .arg("-p")
                .arg(format!("{}", port))
                .arg("-m")
                .arg(
                    diem_root_key_path
                        .canonicalize()
                        .expect("Unable to get canonical path of diem root key file")
                        .to_str()
                        .unwrap(),
                )
                .arg("-c")
                .arg(ChainId::test().id().to_string())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .with_context(|| {
                    format!(
                        "Error launching faucet process with binary: {:?}",
                        faucet_bin_path
                    )
                })
                .expect("Failed to spawn faucet process"),
        }
    }

    pub fn mint_url(&self) -> String {
        return format!("http://localhost:{}/mint", self.port);
    }

    pub fn health_check_url(&self) -> Url {
        Url::parse(format!("http://localhost:{}/-/healthy", self.port).as_str()).unwrap()
    }

    pub fn wait_for_connectivity(&self) -> Result<()> {
        let client = blocking::Client::new();
        let num_attempts = 60;

        for i in 0..num_attempts {
            println!("Wait for faucet connectivity attempt: {}", i);
            let resp = client.get(self.health_check_url()).send();

            if let Ok(ret) = resp {
                if let StatusCode::OK = ret.status() {
                    println!("{}", ret.text()?);
                    return Ok(());
                }
            }
            ::std::thread::sleep(::std::time::Duration::from_millis(500));
        }
        Err(format_err!("Faucet launch failed"))
    }
}
