// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use libra_crypto::{ed25519::*, test_utils::KeyPair};
use libra_temppath::TempPath;
use rand::{rngs::OsRng, Rng, SeedableRng};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn create_faucet_key_file(output_file: &str) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    let output_file_path = Path::new(&output_file);

    if output_file_path.exists() && !output_file_path.is_file() {
        panic!("Specified output file path is a directory");
    }

    let mut seed_rng = OsRng::new().expect("can't access OsRng");
    let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());

    let (private_key, _) = compat::generate_keypair(&mut rng);
    let keypair = KeyPair::from(private_key);

    // Write to disk
    let encoded = lcs::to_bytes(&keypair).expect("Unable to serialize keys");
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
    lcs::from_bytes(&fs::read(path)?[..]).map_err(Into::into)
}

/// Returns the generated or loaded keypair, the path to the file where this keypair is saved,
/// and a reference to the temp directory that was possibly created (a handle so that
/// it doesn't go out of scope)
pub fn load_faucet_key_or_create_default(
    file_path: Option<String>,
) -> (
    KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    String,
    Option<TempPath>,
) {
    // If there is already a faucet key file, then open it and parse the keypair.  If there
    // isn't one, then create a temp directory and generate the keypair
    if let Some(faucet_account_file) = file_path {
        match load_key_from_file(faucet_account_file.clone()) {
            Ok(keypair) => (keypair, faucet_account_file, None),
            Err(e) => {
                panic!(
                    "Unable to read faucet account file: {}, {}",
                    faucet_account_file, e
                );
            }
        }
    } else {
        // Generate keypair in temp directory
        let tmp_dir = TempPath::new();
        tmp_dir
            .create_as_dir()
            .expect("Unable to create temp dir for faucet keypair");
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_test_create_and_load_key() {
        // Create key pair and file
        let (key_pair, key_file_path, _handle) = load_faucet_key_or_create_default(None);

        // Load previously created key from file
        let (loaded_key_pair, _, _) = load_faucet_key_or_create_default(Some(key_file_path));

        // Compare loaded key with created key
        assert_eq!(key_pair, loaded_key_pair);
    }
}
