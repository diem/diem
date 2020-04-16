// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use rand::{rngs::OsRng, Rng, SeedableRng};
use std::{
    fs::{self, File},
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

    let encoded = lcs::to_bytes(&key).expect("Unable to serialize keys");
    let mut file =
        File::create(output_file_path).expect("Unable to create/truncate file at specified path");
    file.write_all(&encoded)
        .expect("Unable to write key to file at specified path");
    key
}

pub fn load_key<P: AsRef<Path>>(input_file: P) -> Ed25519PrivateKey {
    let input_file_path = input_file.as_ref();
    let data = fs::read(input_file_path).expect("Unable to read key at the specified path");
    lcs::from_bytes(&data).expect("Unable to parse key")
}

#[cfg(test)]
mod test {
    use super::*;
    use libra_temppath::TempPath;

    #[test]
    fn verify_test_create_and_load_key() {
        let path = TempPath::new();
        path.create_as_file().unwrap();
        let out_key = generate_and_save_key(path.path());
        let in_key = load_key(path.path());
        assert_eq!(out_key, in_key);
    }
}
