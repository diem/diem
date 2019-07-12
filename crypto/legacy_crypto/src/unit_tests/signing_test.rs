// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{hash::HashValue, signing::*, utils::*};
use bincode::{deserialize, serialize};
use core::ops::{Index, IndexMut};
use proptest::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use test::Bencher;

#[test]
fn test_generate_and_encode_keypair() {
    let (pub_key_human, pub_key_serialized, priv_key_serialized) = generate_and_encode_keypair();
    assert!(!pub_key_human.is_empty());
    assert!(!pub_key_serialized.is_empty());
    assert!(!priv_key_serialized.is_empty());

    let public_key_out = from_encoded_string::<PublicKey>(pub_key_serialized);
    let _private_key_out = from_encoded_string::<PrivateKey>(priv_key_serialized);

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
    let (private_key, public_key) = derive_keypair_from_seed(salt, &seed, info);
    let hash = HashValue::zero();
    let signature = sign_message(hash, &private_key).unwrap();
    assert!(verify_signature(hash, &signature, &public_key).is_ok());

    // HKDF with salt and info.
    let raw_bytes = [2u8; 10];
    let salt = Some(&raw_bytes[0..4]);
    let seed = [3u8; 32];
    let info = Some(&raw_bytes[4..10]);

    let (private_key1, public_key1) = derive_keypair_from_seed(salt, &seed, info);
    let (_, public_key2) = derive_keypair_from_seed(salt, &seed, info);
    assert_eq!(public_key1, public_key2); // Ensure determinism.

    let hash = HashValue::zero();
    let signature = sign_message(hash, &private_key1).unwrap();
    assert!(verify_signature(hash, &signature, &public_key1).is_ok());
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

#[bench]
pub fn bench_sign(bh: &mut Bencher) {
    let (private_key, _) = generate_keypair();
    let hash = HashValue::zero();

    bh.iter(|| {
        let _ = sign_message(hash, &private_key);
    });
}

#[bench]
pub fn bench_verify(bh: &mut Bencher) {
    let (private_key, public_key) = generate_keypair();
    let hash = HashValue::zero();
    let signature = sign_message(hash, &private_key).unwrap();

    bh.iter(|| {
        verify_signature(hash, &signature, &public_key).unwrap();
    });
}

proptest! {
    #[test]
    fn test_keys_encode((private_key, public_key) in keypair_strategy()) {
        {
            let serialized = serialize(&private_key).unwrap();
            let encoded = ::hex::encode(&serialized);
            let decoded = from_encoded_string::<PrivateKey>(encoded);
            prop_assert_eq!(private_key, decoded);
        }
        {
            let serialized = serialize(&public_key).unwrap();
            let encoded = ::hex::encode(&serialized);
            let decoded = from_encoded_string::<PublicKey>(encoded);
            prop_assert_eq!(public_key, decoded);
        }
    }

    #[test]
    fn test_keys_serde((private_key, public_key) in keypair_strategy()) {
        {
            let serialized = serialize(&private_key).unwrap();
            let deserialized = deserialize::<PrivateKey>(&serialized).unwrap();
            prop_assert_eq!(private_key, deserialized);
        }
        {
            let serialized = serialize(&public_key).unwrap();
            let deserialized = deserialize::<PublicKey>(&serialized).unwrap();
            prop_assert_eq!(public_key, deserialized);
        }
    }

    #[test]
    fn test_signature_serde(
        hash in any::<HashValue>(),
        (private_key, _public_key) in keypair_strategy()
    ) {
        let signature = sign_message(hash, &private_key).unwrap();
        let serialized = serialize(&signature).unwrap();
        let deserialized = deserialize::<Signature>(&serialized).unwrap();
        assert_eq!(signature, deserialized);
    }

    #[test]
    fn test_sign_and_verify(
        hash in any::<HashValue>(),
        (private_key, public_key) in keypair_strategy()
    ) {
        let signature = sign_message(hash, &private_key).unwrap();
        prop_assert!(verify_signature(hash, &signature, &public_key).is_ok());
    }

    // Check for canonical s and malleable signatures.
    #[test]
    fn test_signature_malleability(
        hash in any::<HashValue>(),
        (private_key, public_key) in keypair_strategy()
    ) {
        // ed25519-dalek signing ensures a canonical s value.
        let signature = sign_message(hash, &private_key).unwrap();
        // Canonical signatures can be verified as expected.
        prop_assert!(verify_signature(hash, &signature, &public_key).is_ok());

        let mut serialized = signature.to_compact();

        let mut r_bits: [u8; 32] = [0u8; 32];
        r_bits.copy_from_slice(&serialized[..32]);

        let mut s_bits: [u8; 32] = [0u8; 32];
        s_bits.copy_from_slice(&serialized[32..]);

        // ed25519-dalek signing ensures a canonical s value.
        let s = Scalar52::from_bytes(&s_bits);

        // adding L (order of the base point) so that s + L > L
        let malleable_s = Scalar52::add(&s, &L);
        let malleable_s_bits = malleable_s.to_bytes();
        // Update the signature (the s part); the signature gets not canonical.
        serialized[32..].copy_from_slice(&malleable_s_bits);

        let non_canonical_sig = Signature::from_compact(&serialized).unwrap();

        // Check that malleable signatures will pass verification and deserialization in dalek.
        // Construct the corresponding dalek public key.
        let dalek_public_key = ed25519_dalek::PublicKey::from_bytes(&public_key.to_slice()).unwrap();

        // Construct the corresponding dalek Signature. This signature is not canonical.
        let dalek_sig = ed25519_dalek::Signature::from_bytes(&non_canonical_sig.to_compact());

        // ed25519_dalek will verify malleable signatures as valid.
        prop_assert!(dalek_public_key.verify(hash.as_ref(), &dalek_sig.unwrap()).is_ok());

        // Malleable signatures will fail to verify in our implementation, even if for some reason
        // we received one. We detect non canonical signatures.
        prop_assert!(verify_signature(hash, &non_canonical_sig, &public_key).is_err());
    }
}

/// The `Scalar52` struct represents an element in
/// ℤ/ℓℤ as 5 52-bit limbs.
struct Scalar52(pub [u64; 5]);

/// `L` is the order of base point, i.e. 2^252 + 27742317777372353535851937790883648493
const L: Scalar52 = Scalar52([
    0x0002_631a_5cf5_d3ed,
    0x000d_ea2f_79cd_6581,
    0x0000_0000_0014_def9,
    0x0000_0000_0000_0000,
    0x0000_1000_0000_0000,
]);

impl Scalar52 {
    /// Return the zero scalar
    fn zero() -> Scalar52 {
        Scalar52([0, 0, 0, 0, 0])
    }

    /// Unpack a 256 bit scalar into 5 52-bit limbs.
    fn from_bytes(bytes: &[u8; 32]) -> Scalar52 {
        let mut words = [0u64; 4];
        for i in 0..4 {
            for j in 0..8 {
                words[i] |= u64::from(bytes[(i * 8) + j]) << (j * 8);
            }
        }

        let mask = (1u64 << 52) - 1;
        let top_mask = (1u64 << 48) - 1;
        let mut s = Scalar52::zero();

        s[0] = words[0] & mask;
        s[1] = ((words[0] >> 52) | (words[1] << 12)) & mask;
        s[2] = ((words[1] >> 40) | (words[2] << 24)) & mask;
        s[3] = ((words[2] >> 28) | (words[3] << 36)) & mask;
        s[4] = (words[3] >> 16) & top_mask;

        s
    }

    /// Pack the limbs of this `Scalar52` into 32 bytes
    fn to_bytes(&self) -> [u8; 32] {
        let mut s = [0u8; 32];

        s[0] = self.0[0] as u8;
        s[1] = (self.0[0] >> 8) as u8;
        s[2] = (self.0[0] >> 16) as u8;
        s[3] = (self.0[0] >> 24) as u8;
        s[4] = (self.0[0] >> 32) as u8;
        s[5] = (self.0[0] >> 40) as u8;
        s[6] = ((self.0[0] >> 48) | (self.0[1] << 4)) as u8;
        s[7] = (self.0[1] >> 4) as u8;
        s[8] = (self.0[1] >> 12) as u8;
        s[9] = (self.0[1] >> 20) as u8;
        s[10] = (self.0[1] >> 28) as u8;
        s[11] = (self.0[1] >> 36) as u8;
        s[12] = (self.0[1] >> 44) as u8;
        s[13] = self.0[2] as u8;
        s[14] = (self.0[2] >> 8) as u8;
        s[15] = (self.0[2] >> 16) as u8;
        s[16] = (self.0[2] >> 24) as u8;
        s[17] = (self.0[2] >> 32) as u8;
        s[18] = (self.0[2] >> 40) as u8;
        s[19] = ((self.0[2] >> 48) | (self.0[3] << 4)) as u8;
        s[20] = (self.0[3] >> 4) as u8;
        s[21] = (self.0[3] >> 12) as u8;
        s[22] = (self.0[3] >> 20) as u8;
        s[23] = (self.0[3] >> 28) as u8;
        s[24] = (self.0[3] >> 36) as u8;
        s[25] = (self.0[3] >> 44) as u8;
        s[26] = self.0[4] as u8;
        s[27] = (self.0[4] >> 8) as u8;
        s[28] = (self.0[4] >> 16) as u8;
        s[29] = (self.0[4] >> 24) as u8;
        s[30] = (self.0[4] >> 32) as u8;
        s[31] = (self.0[4] >> 40) as u8;

        s
    }

    /// Compute `a + b` (without mod ℓ)
    fn add(a: &Scalar52, b: &Scalar52) -> Scalar52 {
        let mut sum = Scalar52::zero();
        let mask = (1u64 << 52) - 1;

        // a + b
        let mut carry: u64 = 0;
        for i in 0..5 {
            carry = a[i] + b[i] + (carry >> 52);
            sum[i] = carry & mask;
        }

        sum
    }
}

impl Index<usize> for Scalar52 {
    type Output = u64;
    fn index(&self, _index: usize) -> &u64 {
        &(self.0[_index])
    }
}

impl IndexMut<usize> for Scalar52 {
    fn index_mut(&mut self, _index: usize) -> &mut u64 {
        &mut (self.0[_index])
    }
}
