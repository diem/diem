// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{utils::from_encoded_string, x25519::*};
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_generate_and_encode_keypair() {
    let (pub_key_human, pub_key_serialized, priv_key_serialized) = generate_and_encode_keypair();
    assert!(!pub_key_human.is_empty());
    assert!(!pub_key_serialized.is_empty());
    assert!(!priv_key_serialized.is_empty());

    let public_key_out = from_encoded_string::<X25519PublicKey>(pub_key_serialized);
    let public_key_human_out = format!("{:?}", public_key_out);
    assert_eq!(pub_key_human, public_key_human_out)
}

#[test]
fn test_default_key_pair() {
    let mut seed: [u8; 32] = [0u8; 32];
    seed[..4].copy_from_slice(&[1, 2, 3, 4]);
    let keypair1;
    let keypair2;
    {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        keypair1 = generate_keypair_for_testing(&mut rng);
    }
    {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        keypair2 = generate_keypair_for_testing(&mut rng);
    }
    assert_eq!(keypair1.1, keypair2.1);
}

#[test]
fn test_hkdf_key_pair() {
    // HKDF without salt and info.
    let salt = None;
    let seed = [0u8; 32];
    let info = None;
    let (_, public_key1) = derive_keypair_from_seed(salt, &seed, info);
    let (_, public_key2) = derive_keypair_from_seed(salt, &seed, info);
    assert_eq!(public_key1, public_key2);

    // HKDF with salt and info.
    let raw_bytes = [2u8; 10];
    let salt = Some(&raw_bytes[0..4]);
    let seed = [3u8; 32];
    let info = Some(&raw_bytes[4..10]);
    let (_, public_key1) = derive_keypair_from_seed(salt, &seed, info);
    let (_, public_key2) = derive_keypair_from_seed(salt, &seed, info);
    assert_eq!(public_key1, public_key2);
}

#[test]
fn test_generate_key_pair_with_seed() {
    let salt = &b"some salt"[..];
    // In production, ensure seed has at least 256 bits of entropy.
    let seed = [5u8; 32]; // seed is denoted as IKM in HKDF RFC 5869.
    let info = &b"some app info"[..];
    let (_, public_key1) = generate_keypair_hybrid(Some(salt), &seed, Some(info));
    let (_, public_key2) = generate_keypair_hybrid(Some(salt), &seed, Some(info));
    assert_ne!(public_key1, public_key2);

    // Ensure that the deterministic generate_keypair_from_seed returns a completely different key.
    let (_, public_key3) = derive_keypair_from_seed(Some(salt), &seed, Some(info));
    assert_ne!(public_key3, public_key1);
    assert_ne!(public_key3, public_key2);
}
