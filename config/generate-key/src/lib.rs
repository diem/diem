// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use rand::{rngs::OsRng, Rng, SeedableRng};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn generate_key() -> Ed25519PrivateKey {
    let mut seed_rng = OsRng;
    let mut rng = rand::rngs::StdRng::from_seed(seed_rng.gen());
    Ed25519PrivateKey::generate(&mut rng)
}

pub fn generate_and_save_key<P: AsRef<Path>>(output_file: P) -> Ed25519PrivateKey {
    let key = generate_key();
    save_key(key, output_file)
}

pub fn save_key<P: AsRef<Path>>(key: Ed25519PrivateKey, output_file: P) -> Ed25519PrivateKey {
    let output_file_path = output_file.as_ref();
    if output_file_path.exists() && !output_file_path.is_file() {
        panic!("Specified output file path is a directory");
    }

    let encoded = bcs::to_bytes(&key).expect("Unable to serialize keys");
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    // The key file should only be readable by its owner.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts
        .open(output_file_path)
        .expect("Unable to create/truncate file at specified path");
    file.write_all(&encoded)
        .expect("Unable to write key to file at specified path");
    key
}

pub fn load_key<P: AsRef<Path>>(input_file: P) -> Ed25519PrivateKey {
    let input_file_path = input_file.as_ref();
    let data = fs::read(input_file_path).expect("Unable to read key at the specified path");
    bcs::from_bytes(&data).expect("Unable to parse key")
}

#[cfg(test)]
mod test {
    use super::*;
    use diem_temppath::TempPath;

    #[test]
    fn verify_test_create_and_load_key() {
        let path = TempPath::new();
        path.create_as_file().unwrap();
        let out_key = generate_and_save_key(path.path());
        let in_key = load_key(path.path());
        assert_eq!(out_key, in_key);
    }

    #[cfg(unix)]
    #[test]
    fn verify_saved_key_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let path = TempPath::new();
        generate_and_save_key(path.path());
        let mode = fs::metadata(path.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
