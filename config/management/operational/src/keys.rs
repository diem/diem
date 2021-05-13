// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_client::AccountAddress;
use diem_config::config::{Peer, PeerRole};
use diem_crypto::{
    ed25519::Ed25519PrivateKey, x25519, x25519::PUBLIC_KEY_SIZE, Uniform, ValidCryptoMaterial,
    ValidCryptoMaterialStringExt,
};
use diem_global_constants::OWNER_ACCOUNT;
use diem_management::{
    config::ConfigPath, error::Error, secure_backend::ValidatorBackend, storage::StorageWrapper,
};
use diem_types::{account_address::from_identity_public_key, PeerId};
use rand::SeedableRng;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{read, File},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GenerateKey {
    /// Location to store the key
    #[structopt(long)]
    key_file: PathBuf,
    #[structopt(long)]
    key_type: KeyType,
    #[structopt(long)]
    encoding: EncodingType,
}

#[derive(Clone, Copy, Debug, StructOpt)]
pub enum KeyType {
    Ed25519,
    X25519,
}

impl FromStr for KeyType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ed25519" => Ok(KeyType::Ed25519),
            "x25519" => Ok(KeyType::X25519),
            _ => Err("Invalid key type"),
        }
    }
}

#[derive(Clone, Copy, Debug, StructOpt)]
pub enum EncodingType {
    BCS,
    Hex,
    Base64,
}

impl FromStr for EncodingType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hex" => Ok(EncodingType::Hex),
            "bcs" => Ok(EncodingType::BCS),
            "base64" => Ok(EncodingType::Base64),
            _ => Err("Invalid encoding type"),
        }
    }
}

impl GenerateKey {
    pub fn execute(self) -> Result<(), Error> {
        let ed25519_key = &generate_ed25519_key();
        match self.key_type {
            KeyType::X25519 => {
                let key = x25519::PrivateKey::from_ed25519_private_bytes(&ed25519_key.to_bytes())
                    .map_err(|err| Error::UnexpectedError(err.to_string()))?;
                save_key(
                    &key,
                    "x22519 PrivateKey",
                    self.key_file.as_path(),
                    self.encoding,
                )
            }
            KeyType::Ed25519 => save_key(
                ed25519_key,
                "ed22519 PrivateKey",
                self.key_file.as_path(),
                self.encoding,
            ),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct ExtractPeersFromKeys {
    #[structopt(long, parse(try_from_str = parse_public_keys_hex))]
    keys: HashSet<x25519::PublicKey>,
    #[structopt(long)]
    output_file: Option<PathBuf>,
}

fn parse_public_key_hex(src: &str) -> Result<x25519::PublicKey, Error> {
    let input = src.trim();
    static HEX_SIZE: usize = PUBLIC_KEY_SIZE * 2;
    if input.len() != HEX_SIZE {
        return Err(Error::CommandArgumentError(
            "Invalid public key length, must be 64 hex characters".to_string(),
        ));
    }
    x25519::PublicKey::from_encoded_string(input)
        .map_err(|err| Error::UnableToParse("Key", err.to_string()))
}

fn parse_public_keys_hex(src: &str) -> Result<HashSet<x25519::PublicKey>, Error> {
    let input = src.trim();

    let strings: Vec<_> = input.split(',').collect();
    let mut keys: HashSet<_> = HashSet::new();
    for str in strings {
        keys.insert(parse_public_key_hex(str)?);
    }

    Ok(keys)
}

impl ExtractPeersFromKeys {
    pub fn execute(&self) -> Result<HashMap<PeerId, Peer>, Error> {
        let map = self
            .keys
            .iter()
            .map(|key| peer_from_public_key(*key))
            .collect();
        if let Some(output_file) = self.output_file.as_ref() {
            save_to_yaml(output_file, "Peers", &map)?;
        }
        Ok(map)
    }
}

#[derive(Debug, StructOpt)]
pub struct ExtractPeerFromFile {
    /// Location to read the key
    #[structopt(long)]
    key_file: PathBuf,
    #[structopt(long)]
    output_file: Option<PathBuf>,
    #[structopt(long)]
    encoding: EncodingType,
}

impl ExtractPeerFromFile {
    pub fn execute(self) -> Result<HashMap<PeerId, Peer>, Error> {
        let private_key = load_key::<x25519::PrivateKey>(self.key_file, self.encoding)?;
        let (peer_id, peer) = peer_from_public_key(private_key.public_key());
        let mut map = HashMap::new();
        map.insert(peer_id, peer);

        if let Some(output_file) = self.output_file.as_ref() {
            save_to_yaml(output_file, "Peer", &map)?;
        }
        Ok(map)
    }
}

#[derive(Debug, StructOpt)]
pub struct ExtractPeerFromStorage {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The keys name in storage
    #[structopt(long)]
    key_name: String,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(long)]
    output_file: Option<PathBuf>,
}

impl ExtractPeerFromStorage {
    pub fn execute(self) -> Result<HashMap<PeerId, Peer>, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let key_name = Box::leak(self.key_name.into_boxed_str());
        let storage = config.validator_backend();
        let peer_id = storage.account_address(OWNER_ACCOUNT)?;
        let public_key = storage.x25519_public_from_private(key_name)?;
        let (_, peer) = peer_from_public_key(public_key);
        let mut map = HashMap::new();
        map.insert(peer_id, peer);

        if let Some(output_file) = self.output_file.as_ref() {
            save_to_yaml(output_file, "Peer", &map)?;
        }
        Ok(map)
    }
}

fn peer_from_public_key(public_key: x25519::PublicKey) -> (AccountAddress, Peer) {
    let peer_id = from_identity_public_key(public_key);
    let mut public_keys = HashSet::new();
    public_keys.insert(public_key);
    (
        peer_id,
        Peer::new(Vec::new(), public_keys, PeerRole::Downstream),
    )
}

fn generate_ed25519_key() -> Ed25519PrivateKey {
    let mut rng = rand::rngs::StdRng::from_entropy();
    Ed25519PrivateKey::generate(&mut rng)
}

#[derive(Debug, StructOpt)]
struct ExtractKey {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The keys name in storage
    #[structopt(long)]
    key_name: String,
    /// Location to store the key
    #[structopt(long)]
    key_file: PathBuf,
    #[structopt(long)]
    encoding: Option<EncodingType>,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl ExtractKey {
    fn save_key<
        Key: ValidCryptoMaterial,
        F: FnOnce(StorageWrapper, &'static str) -> Result<Key, Error>,
    >(
        &self,
        load_key: F,
    ) -> Result<(), Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let encoding = self.encoding.unwrap_or(EncodingType::BCS);
        let storage = config.validator_backend();
        let key_name = Box::leak(self.key_name.clone().into_boxed_str());
        let key = load_key(storage, key_name)?;
        save_key(&key, key_name, &self.key_file, encoding)
    }
}

#[derive(Debug, StructOpt)]
pub struct ExtractPublicKey {
    #[structopt(flatten)]
    extract_key: ExtractKey,
    #[structopt(long)]
    key_type: Option<KeyType>,
}

impl ExtractPublicKey {
    pub fn execute(self) -> Result<(), Error> {
        match self.key_type {
            Some(KeyType::X25519) => self
                .extract_key
                .save_key(|storage, key_name| storage.x25519_public_from_private(key_name)),
            _ => self
                .extract_key
                .save_key(|storage, key_name| storage.ed25519_public_from_private(key_name)),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct ExtractPrivateKey {
    #[structopt(flatten)]
    extract_key: ExtractKey,
    #[structopt(long)]
    key_type: Option<KeyType>,
}

impl ExtractPrivateKey {
    pub fn execute(self) -> Result<(), Error> {
        match self.key_type {
            Some(KeyType::X25519) => self
                .extract_key
                .save_key(|storage, key_name| storage.x25519_private(key_name)),
            _ => self
                .extract_key
                .save_key(|storage, key_name| storage.ed25519_private(key_name)),
        }
    }
}

/// Loads a key to a file hex string encoded
pub fn load_key<Key: ValidCryptoMaterial>(
    path: PathBuf,
    encoding: EncodingType,
) -> Result<Key, Error> {
    let data = read(&path).map_err(|err| {
        Error::UnableToReadFile(path.to_str().unwrap().to_string(), err.to_string())
    })?;

    match encoding {
        EncodingType::BCS => {
            bcs::from_bytes(&data).map_err(|err| Error::BCS("Key".to_string(), err))
        }
        EncodingType::Hex => {
            let hex_string = String::from_utf8(data).unwrap();
            Key::from_encoded_string(&hex_string)
                .map_err(|err| Error::UnableToParse("Key", err.to_string()))
        }
        EncodingType::Base64 => {
            let bytes =
                base64::decode(data).map_err(|err| Error::UnableToParse("Key", err.to_string()))?;
            Key::try_from(bytes.as_slice())
                .map_err(|err| Error::UnexpectedError(format!("Failed to parse key {}", err)))
        }
    }
}

/// Saves a key to a file encoded in a string
pub fn save_key<Key: ValidCryptoMaterial>(
    key: &Key,
    key_name: &'static str,
    path: &Path,
    encoding: EncodingType,
) -> Result<(), Error> {
    let encoded = match encoding {
        EncodingType::Hex => hex::encode_upper(key.to_bytes()).into_bytes(),
        EncodingType::BCS => {
            bcs::to_bytes(key).map_err(|err| Error::BCS(key_name.to_string(), err))?
        }
        EncodingType::Base64 => base64::encode(key.to_bytes()).into_bytes(),
    };

    write_file(path, key_name, encoded.as_slice())
}

fn save_to_yaml<T: Serialize>(path: &Path, input_name: &str, item: &T) -> Result<(), Error> {
    let yaml =
        serde_yaml::to_string(item).map_err(|err| Error::UnexpectedError(err.to_string()))?;
    write_file(path, input_name, yaml.as_bytes())
}

fn write_file(path: &Path, input_name: &str, contents: &[u8]) -> Result<(), Error> {
    let mut file = File::create(path).map_err(|e| Error::IO(input_name.to_string(), e))?;
    file.write_all(contents)
        .map_err(|e| Error::IO(input_name.to_string(), e))?;
    Ok(())
}
