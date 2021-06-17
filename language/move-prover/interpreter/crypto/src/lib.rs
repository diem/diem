// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use ed25519_dalek::{
    ed25519::signature::Signature, PublicKey as Ed25519PublicKey, Signature as Ed25519Signature,
};

use sha2::{Digest, Sha256};
use sha3::Sha3_256;

// Hash functions
pub fn sha2_256_of(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(&bytes).to_vec()
}

pub fn sha3_256_of(bytes: &[u8]) -> Vec<u8> {
    Sha3_256::digest(&bytes).to_vec()
}

// Ed25519
pub fn ed25519_deserialize_public_key(bytes: &[u8]) -> Result<Ed25519PublicKey> {
    Ok(Ed25519PublicKey::from_bytes(bytes)?)
}

pub fn ed25519_deserialize_signature(bytes: &[u8]) -> Result<Ed25519Signature> {
    Ok(Ed25519Signature::from_bytes(bytes)?)
}

pub fn ed25519_verify_signature(
    key: &Ed25519PublicKey,
    sig: &Ed25519Signature,
    msg: &[u8],
) -> Result<()> {
    Ok(key.verify_strict(msg, sig)?)
}
