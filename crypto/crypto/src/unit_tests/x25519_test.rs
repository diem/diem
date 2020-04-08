// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{traits::*, x25519::*};
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_default_key_pair() {
    let mut seed: [u8; 32] = [0u8; 32];
    seed[..4].copy_from_slice(&[1, 2, 3, 4]);
    let public_key1: X25519StaticPublicKey;
    let public_key2: X25519StaticPublicKey;
    {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let private_key1 = X25519StaticPrivateKey::generate(&mut rng);
        public_key1 = (&private_key1).into();
    }
    {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let private_key2 = X25519StaticPrivateKey::generate(&mut rng);
        public_key2 = (&private_key2).into();
    }
    assert_eq!(public_key1, public_key2);
}

#[test]
fn test_hkdf_key_pair() {
    // HKDF without salt and info.
    let salt = None;
    let seed = [0u8; 32];
    let info = None;
    let (_, public_key1) = X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, info);
    let (_, public_key2) = X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, info);
    assert_eq!(public_key1, public_key2);

    // HKDF with salt and info.
    let raw_bytes = [2u8; 10];
    let salt = Some(&raw_bytes[..4]);
    let seed = [3u8; 32];
    let info = Some(&raw_bytes[4..10]);
    let (_, public_key1) = X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, info);
    let (_, public_key2) = X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, info);
    assert_eq!(public_key1, public_key2);
}

#[test]
fn test_generate_key_pair_with_seed() {
    let salt = &b"some salt"[..];
    // In production, ensure seed has at least 256 bits of entropy.
    let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
    let info = &b"some app info"[..];
    let (_, public_key1) =
        X25519StaticPrivateKey::generate_keypair_hybrid(Some(salt), &seed, Some(info));
    let (_, public_key2) =
        X25519StaticPrivateKey::generate_keypair_hybrid(Some(salt), &seed, Some(info));
    assert_ne!(public_key1, public_key2);

    // Ensure that the deterministic generate_keypair_from_seed returns a completely different key.
    let (_, public_key3) =
        X25519StaticPrivateKey::derive_keypair_from_seed(Some(salt), &seed, Some(info));
    assert_ne!(public_key3, public_key1);
    assert_ne!(public_key3, public_key2);
}

#[test]
fn test_serialize_deserialize() {
    let seed: [u8; 32] = [0u8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let private_key = X25519StaticPrivateKey::generate(&mut rng);
    let public_key: X25519StaticPublicKey = (&private_key).into();

    let serialized = lcs::to_bytes(&private_key).unwrap();
    let deserialized: X25519StaticPrivateKey = lcs::from_bytes(&serialized).unwrap();

    assert_eq!(public_key, (&deserialized).into());
}
