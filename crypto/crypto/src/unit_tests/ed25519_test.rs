// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ed25519::{
        Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PRIVATE_KEY_LENGTH,
        ED25519_PUBLIC_KEY_LENGTH, ED25519_SIGNATURE_LENGTH,
    },
    test_utils::uniform_keypair_strategy,
    traits::*,
};

use core::{
    convert::TryFrom,
    ops::{Index, IndexMut},
};

use crate::hash::HashValue;
use proptest::{collection::vec, prelude::*};

proptest! {
    #[test]
    fn test_keys_encode(keypair in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>()) {
        {
            let encoded = keypair.private_key.to_encoded_string().unwrap();
             // Hex encoding of a 32-bytes key is 64 (2 x 32) characters.
            prop_assert_eq!(2 * ED25519_PRIVATE_KEY_LENGTH, encoded.len());
            let decoded = Ed25519PrivateKey::from_encoded_string(&encoded);
            prop_assert_eq!(Some(keypair.private_key), decoded.ok());
        }
        {
            let encoded = keypair.public_key.to_encoded_string().unwrap();
            // Hex encoding of a 32-bytes key is 64 (2 x 32) characters.
            prop_assert_eq!(2 * ED25519_PUBLIC_KEY_LENGTH, encoded.len());
            let decoded = Ed25519PublicKey::from_encoded_string(&encoded);
            prop_assert_eq!(Some(keypair.public_key), decoded.ok());
        }
    }

    #[test]
    fn test_batch_verify(
        hash in any::<HashValue>(),
        keypairs in proptest::array::uniform10(uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>())) {
        let mut signatures: Vec<(Ed25519PublicKey, Ed25519Signature)> = keypairs.iter().map(|keypair| {
            (keypair.public_key.clone(), keypair.private_key.sign_message(&hash))
        }).collect();
        prop_assert!(Ed25519Signature::batch_verify_signatures(&hash, signatures.clone()).is_ok());
        // We swap message and signature for the last element,
        // resulting in an incorrect signature
        let (key, _sig) = signatures.pop().unwrap();
        let other_sig = signatures.last().unwrap().clone().1;
        signatures.push((key, other_sig));
        prop_assert!(Ed25519Signature::batch_verify_signatures(&hash, signatures).is_err());
    }

    #[test]
    fn test_keys_custom_serialisation(
        keypair in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>()
    ) {
        {
            let serialized: &[u8] = &(keypair.private_key.to_bytes());
            prop_assert_eq!(ED25519_PRIVATE_KEY_LENGTH, serialized.len());
            let deserialized = Ed25519PrivateKey::try_from(serialized);
            prop_assert_eq!(Some(keypair.private_key), deserialized.ok());
        }
        {
            let serialized: &[u8] = &(keypair.public_key.to_bytes());
            prop_assert_eq!(ED25519_PUBLIC_KEY_LENGTH, serialized.len());
            let deserialized = Ed25519PublicKey::try_from(serialized);
            prop_assert_eq!(Some(keypair.public_key), deserialized.ok());
        }
    }

    #[test]
    fn test_signature_verification_custom_serialisation(
        hash in any::<HashValue>(),
        keypair in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>()
    ) {
        let signature = keypair.private_key.sign_message(&hash);
        let serialized: &[u8] = &(signature.to_bytes());
        prop_assert_eq!(ED25519_SIGNATURE_LENGTH, serialized.len());
        let deserialized = Ed25519Signature::try_from(serialized).unwrap();
        prop_assert!(keypair.public_key.verify_signature(&hash, &deserialized).is_ok());
    }

    #[test]
    fn test_signature_verification_from_arbitrary(
        // this should be > 64 bits to go over the length of a default hash
        msg in vec(proptest::num::u8::ANY, 1..128),
        keypair in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>()
    ) {
        let signature = keypair.private_key.sign_arbitrary_message(&msg);
        let serialized: &[u8] = &(signature.to_bytes());
        prop_assert_eq!(ED25519_SIGNATURE_LENGTH, serialized.len());
        let deserialized = Ed25519Signature::try_from(serialized).unwrap();
        prop_assert!(deserialized.verify_arbitrary_msg(&msg, &keypair.public_key).is_ok());
    }

    // Check for canonical S.
    #[test]
    fn test_signature_malleability(
        hash in any::<HashValue>(),
        keypair in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>()
    ) {
        let signature = keypair.private_key.sign_message(&hash);
        let mut serialized = signature.to_bytes();

        let mut r_bytes: [u8; 32] = [0u8; 32];
        r_bytes.copy_from_slice(&serialized[..32]);

        let mut s_bytes: [u8; 32] = [0u8; 32];
        s_bytes.copy_from_slice(&serialized[32..]);

        // ed25519-dalek signing ensures a canonical S value.
        let s = Scalar52::from_bytes(&s_bytes);

        // adding L (order of the base point) so that S + L > L
        let malleable_s = Scalar52::add(&s, &L);
        let malleable_s_bytes = malleable_s.to_bytes();
        // Update the signature (the S part).
        serialized[32..].copy_from_slice(&malleable_s_bytes);

        // Check that malleable signatures will pass verification and deserialization in dalek.
        // Construct the corresponding dalek public key.
        let _dalek_public_key = ed25519_dalek::PublicKey::from_bytes(
            &keypair.public_key.to_bytes()
        ).unwrap();

        // Construct the corresponding dalek Signature. This signature is malleable.
        let dalek_sig = ed25519_dalek::Signature::from_bytes(&serialized);

        // ed25519_dalek will (post 2.0) no longer deserialize the malleable signature. It does detect it.
        prop_assert!(dalek_sig.is_err());

        let serialized_malleable: &[u8] = &serialized;
        // try_from will fail on malleable signatures. We detect malleable signatures
        // during deserialization.
        prop_assert_eq!(
            Ed25519Signature::try_from(serialized_malleable),
            Err(CryptoMaterialError::CanonicalRepresentationError)
        );

        // We expect from_bytes_unchecked deserialization to fail, as dalek checks
        // for signature malleability. This method is pub(crate) and only used for test purposes.
        let sig_unchecked = Ed25519Signature::from_bytes_unchecked(&serialized);
        prop_assert!(sig_unchecked.is_err());

        // Update the signature by setting S = L to make it invalid.
        serialized[32..].copy_from_slice(&L.to_bytes());
        let serialized_malleable_l: &[u8] = &serialized;
        // try_from will fail with CanonicalRepresentationError.
        prop_assert_eq!(
            Ed25519Signature::try_from(serialized_malleable_l),
            Err(CryptoMaterialError::CanonicalRepresentationError)
        );
    }
}

// Test against known small subgroup public keys.
#[test]
fn test_publickey_smallorder() {
    for torsion_point in &EIGHT_TORSION {
        let serialized: &[u8] = torsion_point;
        // We expect from_bytes_unchecked to pass, as it does not validate the key.
        assert!(Ed25519PublicKey::from_bytes_unchecked(serialized).is_ok());
        // from_bytes will fail on invalid key.
        assert_eq!(
            Ed25519PublicKey::try_from(serialized),
            Err(CryptoMaterialError::SmallSubgroupError)
        );
    }
}

// The 8-torsion subgroup E[8].
//
// In the case of Curve25519, it is cyclic; the i-th element of
// the array is [i]P, where P is a point of order 8
// generating E[8].
//
// Thus E[8] is the points indexed by `0,2,4,6`, and
// E[2] is the points indexed by `0,4`.
//
// The following byte arrays have been ported from curve25519-dalek /backend/serial/u64/constants.rs
// and they represent the serialised version of the CompressedEdwardsY points.

const EIGHT_TORSION: [[u8; 32]; 8] = [
    [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    [
        199, 23, 106, 112, 61, 77, 216, 79, 186, 60, 11, 118, 13, 16, 103, 15, 42, 32, 83, 250, 44,
        57, 204, 198, 78, 199, 253, 119, 146, 172, 3, 122,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 128,
    ],
    [
        38, 232, 149, 143, 194, 178, 39, 176, 69, 195, 244, 137, 242, 239, 152, 240, 213, 223, 172,
        5, 211, 198, 51, 57, 177, 56, 2, 136, 109, 83, 252, 5,
    ],
    [
        236, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127,
    ],
    [
        38, 232, 149, 143, 194, 178, 39, 176, 69, 195, 244, 137, 242, 239, 152, 240, 213, 223, 172,
        5, 211, 198, 51, 57, 177, 56, 2, 136, 109, 83, 252, 133,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    [
        199, 23, 106, 112, 61, 77, 216, 79, 186, 60, 11, 118, 13, 16, 103, 15, 42, 32, 83, 250, 44,
        57, 204, 198, 78, 199, 253, 119, 146, 172, 3, 250,
    ],
];

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

    /// Unpack a 32 byte / 256 bit scalar into 5 52-bit limbs.
    fn from_bytes(bytes: &[u8; 32]) -> Scalar52 {
        let mut words = [0u64; 4];
        for i in 0..4 {
            for j in 0..8 {
                words[i] |= u64::from(bytes[(i * 8) + j]) << (j * 8) as u64;
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
