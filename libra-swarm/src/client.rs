// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::waypoint::Waypoint;
use std::{
    io,
    path::Path,
    process::{Child, Command, Output, Stdio},
};

pub struct InteractiveClient {
    client: Option<Child>,
}

impl Drop for InteractiveClient {
    fn drop(&mut self) {
        if self.client.is_none() {
            return;
        }
        // Kill client process if still running.
        let mut client = self.client.take().unwrap();
        match client.try_wait().unwrap() {
            Some(status) => {
                if !status.success() {
                    panic!(
                        "Client terminated with status: {}",
                        status.code().unwrap_or(-1)
                    );
                }
            }
            None => {
                client.kill().unwrap();
            }
        }
    }
}

impl InteractiveClient {
    pub fn new_with_inherit_io(
        port: u16,
        faucet_key_file_path: &Path,
        mnemonic_file_path: &Path,
        waypoint: Waypoint,
    ) -> Self {
        // We need to call canonicalize on the path because we are running client from
        // workspace root and the function calling new_with_inherit_io isn't necessarily
        // running from that location, so if a relative path is passed, it wouldn't work
        // unless we convert it to an absolute path
        Self {
            client: Some(
                Command::new(workspace_builder::get_bin("cli"))
                    .current_dir(workspace_builder::workspace_root())
                    .arg("-u")
                    .arg(format!("http://localhost:{}", port))
                    .arg("-m")
                    .arg(
                        faucet_key_file_path
                            .canonicalize()
                            .expect("Unable to get canonical path of faucet key file")
                            .to_str()
                            .unwrap(),
                    )
                    .arg("-n")
                    .arg(
                        mnemonic_file_path
                            .canonicalize()
                            .expect("Unable to get canonical path of mnemonic file")
                            .to_str()
                            .unwrap(),
                    )
                    .arg("--waypoint")
                    .arg(waypoint.to_string())
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .expect("Failed to spawn client process"),
            ),
        }
    }

    pub fn output(mut self) -> io::Result<Output> {
        self.client.take().unwrap().wait_with_output()
    }
}
