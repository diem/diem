// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bincode::serialize;
use failure::prelude::*;
use nextgen_crypto::{ed25519::*, test_utils::KeyPair};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use tempdir::TempDir;

pub fn create_faucet_key_file(output_file: &str) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    let output_file_path = Path::new(&output_file);

    if output_file_path.exists() && !output_file_path.is_file() {
        panic!("Specified output file path is a directory");
    }

    let (private_key, _) = compat::generate_keypair(None);
    let keypair = KeyPair::from(private_key);

    // Write to disk
    let encoded: Vec<u8> = serialize(&keypair).expect("Unable to serialize keys");
    let mut file =
        File::create(output_file_path).expect("Unable to create/truncate file at specified path");
    file.write_all(&encoded)
        .expect("Unable to write keys to file at specified path");
    keypair
}

/// Tries to load a keypair from the path given as argument
pub fn load_key_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
    bincode::deserialize(&fs::read(path)?[..]).map_err(|b| b.into())
}

/// Returns the generated or loaded keypair, the path to the file where this keypair is saved,
/// and a reference to the temp directory that was possibly created (a handle so that
/// it doesn't go out of scope)
pub fn load_faucet_key_or_create_default(
    file_path: Option<String>,
) -> (
    KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    String,
    Option<TempDir>,
) {
    // If there is already a faucet key file, then open it and parse the keypair.  If there
    // isn't one, then create a temp directory and generate the keypair
    if let Some(faucet_account_file) = file_path {
        match load_key_from_file(faucet_account_file.clone()) {
            Ok(keypair) => (keypair, faucet_account_file.to_string(), None),
            Err(e) => {
                panic!(
                    "Unable to read faucet account file: {}, {}",
                    faucet_account_file, e
                );
            }
        }
    } else {
        // Generate keypair in temp directory
        let tmp_dir =
            TempDir::new("keypair").expect("Unable to create temp dir for faucet keypair");
        let faucet_key_file_path = tmp_dir
            .path()
            .join("temp_faucet_keys")
            .to_str()
            .unwrap()
            .to_string();
        (
            crate::create_faucet_key_file(&faucet_key_file_path),
            faucet_key_file_path,
            Some(tmp_dir),
        )
    }
}
