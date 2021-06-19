// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file duplicates the code in the diem-crypto crate to support
//! - native function implementation for the stackless bytecode interpreter, and
//! - decoupling of Move crates from Diem crates.
//!
//! This is expected to be a temporary solution only. Once we properly
//! restructure the stackless interpreter like what we did to the Move VM,
//! native functions will likely be implemented in a Diem crate (most likely
//! diem-framework) and be passed into the VM for execution. In this way we no
//! longer need to worry about depending on diem-crypto.

use anyhow::{bail, Result};
use ed25519_dalek::{
    ed25519::signature::Signature, PublicKey as Ed25519PublicKey, Signature as Ed25519Signature,
    PUBLIC_KEY_LENGTH as ED25519_PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH as ED25519_SIGNATURE_LENGTH,
};
use sha2::{Digest, Sha256};
use sha3::Sha3_256;
use std::cmp::Ordering;

/// The order of ed25519 as defined in [RFC8032](https://tools.ietf.org/html/rfc8032).
const L: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

// Hash functions
pub fn sha2_256_of(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(&bytes).to_vec()
}

pub fn sha3_256_of(bytes: &[u8]) -> Vec<u8> {
    Sha3_256::digest(&bytes).to_vec()
}

// Ed25519
fn validate_public_key(bytes: &[u8]) -> bool {
    // We need to access the Edwards point which is not directly accessible from
    // ed25519_dalek::PublicKey, so we need to do some custom deserialization.
    if bytes.len() != ED25519_PUBLIC_KEY_LENGTH {
        return false;
    }

    let mut bits = [0u8; ED25519_PUBLIC_KEY_LENGTH];
    bits.copy_from_slice(&bytes[..ED25519_PUBLIC_KEY_LENGTH]);

    let compressed = curve25519_dalek::edwards::CompressedEdwardsY(bits);
    let point = match compressed.decompress() {
        None => return false,
        Some(point) => point,
    };

    // Check if the point lies on a small subgroup. This is required
    // when using curves with a small cofactor (in ed25519, cofactor = 8).
    !point.is_small_order()
}

pub fn ed25519_deserialize_public_key(bytes: &[u8]) -> Result<Ed25519PublicKey> {
    if !validate_public_key(bytes) {
        bail!("Invalid public key bytes");
    }
    Ok(Ed25519PublicKey::from_bytes(bytes)?)
}

fn validate_signature(bytes: &[u8]) -> bool {
    if bytes.len() != ED25519_SIGNATURE_LENGTH {
        return false;
    }
    for i in (0..32).rev() {
        match bytes[32 + i].cmp(&L[i]) {
            Ordering::Less => return true,
            Ordering::Greater => return false,
            _ => (),
        }
    }
    // As this stage S == L which implies a non canonical S.
    false
}

pub fn ed25519_deserialize_signature(bytes: &[u8]) -> Result<Ed25519Signature> {
    if !validate_signature(bytes) {
        bail!("Invalid signature bytes");
    }
    Ok(Ed25519Signature::from_bytes(bytes)?)
}

pub fn ed25519_verify_signature(
    key: &Ed25519PublicKey,
    sig: &Ed25519Signature,
    msg: &[u8],
) -> Result<()> {
    if !validate_public_key(&key.to_bytes()) {
        bail!("Malleable public key");
    }
    if !validate_signature(&sig.to_bytes()) {
        bail!("Malleable signature");
    }
    Ok(key.verify_strict(msg, sig)?)
}
