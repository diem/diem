// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::hkdf::*;
use sha2::{Sha256, Sha512};
use sha3::Sha3_256;

// Testing against sha256 test vectors. Unfortunately the rfc does not provide test vectors for
// sha3 and sha512.
#[test]
fn test_sha256_test_vectors() {
    let tests = test_vectors_sha256();
    for t in tests.iter() {
        let ikm = hex::decode(&t.ikm).unwrap();
        let salt = hex::decode(&t.salt).unwrap();
        let info = hex::decode(&t.info).unwrap();

        let hkdf_extract = Hkdf::<Sha256>::extract(Option::from(&salt[..]), &ikm[..]).unwrap();
        let hkdf_expand = Hkdf::<Sha256>::expand(&hkdf_extract, Some(&info[..]), t.length);

        assert!(hkdf_expand.is_ok());
        assert_eq!(t.prk, hex::encode(hkdf_extract));
        assert_eq!(t.okm, hex::encode(hkdf_expand.unwrap()));
    }
}

// Testing against sha256 test vectors for the extract_then_expand function.
#[test]
fn test_extract_then_expand() {
    let tests = test_vectors_sha256();
    for t in tests.iter() {
        let ikm = hex::decode(&t.ikm).unwrap();
        let salt = hex::decode(&t.salt).unwrap();
        let info = hex::decode(&t.info).unwrap();

        let hkdf_full = Hkdf::<Sha256>::extract_then_expand(
            Option::from(&salt[..]),
            &ikm[..],
            Option::from(&info[..]),
            t.length,
        );

        assert!(hkdf_full.is_ok());
        assert_eq!(t.okm, hex::encode(hkdf_full.unwrap()));
    }
}

#[test]
fn test_sha256_output_length() {
    // According to rfc, max_sha256_length <= 255 * HashLen bytes
    let max_hash_length: usize = 255 * 32; // = 8160

    // We extract once, then we reuse it.
    let hkdf_extract = Hkdf::<Sha256>::extract(None, &[]).unwrap();

    // Test for max allowed (expected to pass)
    let hkdf_expand = Hkdf::<Sha256>::expand(&hkdf_extract, None, max_hash_length);
    assert!(hkdf_expand.is_ok());
    assert_eq!(hkdf_expand.unwrap().len(), max_hash_length);

    // Test for max + 1 (expected to fail)
    let hkdf_expand = Hkdf::<Sha256>::expand(&hkdf_extract, None, max_hash_length + 1);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );

    // Test for 10_000 > max (expected to fail)
    let hkdf_expand = Hkdf::<Sha256>::expand(&hkdf_extract, None, 10_000);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );

    // Test for zero size output (expected to fail)
    let hkdf_expand = Hkdf::<Sha256>::expand(&hkdf_extract, None, 0);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );
}

// FIPS 202 approves HMAC-SHA3 and specifies the block sizes (see top of page 22).
// SP 800-56C approves of HKDF-HMAC-hash as a randomness extractor with any approved hash function.
// But in contrast, I can't find any NIST statement that explicitly approves the use of KMAC
// as a randomness extractor.
// But, it's debatable if this is a pointless construct, as HMAC only exists to cover up weaknesses
// in Merkle-Damgard hashes, but sha3 (and Keccak) are sponge constructions, immune to length
// extension attacks.
#[test]
fn test_sha3_256_output_length() {
    let max_hash_length: usize = 255 * 32; // = 8160

    let hkdf_extract = Hkdf::<Sha3_256>::extract(None, &[]).unwrap();

    // Test for max allowed (expected to pass)
    let hkdf_expand = Hkdf::<Sha3_256>::expand(&hkdf_extract, None, max_hash_length);
    assert!(hkdf_expand.is_ok());
    assert_eq!(hkdf_expand.unwrap().len(), max_hash_length);

    // Test for max + 1 (expected to fail)
    let hkdf_expand = Hkdf::<Sha3_256>::expand(&hkdf_extract, None, max_hash_length + 1);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );

    // Test for 10_000 > max (expected to fail)
    let hkdf_expand = Hkdf::<Sha3_256>::expand(&hkdf_extract, None, 10_000);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );

    // Test for zero size output (expected to fail)
    let hkdf_expand = Hkdf::<Sha3_256>::expand(&hkdf_extract, None, 0);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );
}

#[test]
fn test_sha512_output_length() {
    let max_hash_length: usize = 255 * 64; // = 16320

    let hkdf_extract = Hkdf::<Sha512>::extract(None, &[]).unwrap();

    // Test for max allowed (expected to pass)
    let hkdf_expand = Hkdf::<Sha512>::expand(&hkdf_extract, None, max_hash_length);
    assert!(hkdf_expand.is_ok());
    assert_eq!(hkdf_expand.unwrap().len(), max_hash_length);

    // Test for max + 1 (expected to fail)
    let hkdf_expand = Hkdf::<Sha512>::expand(&hkdf_extract, None, max_hash_length + 1);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );

    // Test for 10_000 > max (expected to fail)
    let hkdf_expand = Hkdf::<Sha512>::expand(&hkdf_extract, None, 20_000);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );

    // Test for zero size output (expected to fail)
    let hkdf_expand = Hkdf::<Sha512>::expand(&hkdf_extract, None, 0);
    assert_eq!(
        hkdf_expand.unwrap_err(),
        HkdfError::InvalidOutputLengthError
    );
}

#[test]
fn test_unsupported_hash_functions() {
    // Test for ripemd160, output_length < 256
    let ripemd160_hkdf = Hkdf::<ripemd160::Ripemd160>::extract(None, &[]);
    assert_eq!(
        ripemd160_hkdf.unwrap_err(),
        HkdfError::NotSupportedHashFunctionError
    );
}

// Test Vectors for sha256 from https://tools.ietf.org/html/rfc5869.
struct Test<'a> {
    ikm: &'a str,
    salt: &'a str,
    info: &'a str,
    length: usize,
    prk: &'a str,
    okm: &'a str,
}

fn test_vectors_sha256<'a>() -> Vec<Test<'a>> {
    vec![
        Test {
            // Test Case 1
            ikm: "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
            salt: "000102030405060708090a0b0c",
            info: "f0f1f2f3f4f5f6f7f8f9",
            length: 42,
            prk: "077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5",
            okm: "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b8\
                  87185865",
        },
        Test {
            // Test Case 2
            ikm: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425\
                  262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b\
                  4c4d4e4f",
            salt: "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f80818283848\
                   5868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aa\
                   abacadaeaf",
            info: "b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d\
                   5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fa\
                   fbfcfdfeff",
            length: 82,
            prk: "06a6b88c5853361a06104c9ceb35b45cef760014904671014a193f40c15fc244",
            okm: "b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c59045a99cac7\
                  827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71cc30c58179ec3e87c14c01d5\
                  c1f3434f1d87",
        },
        Test {
            // Test Case 3
            ikm: "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
            salt: "",
            info: "",
            length: 42,
            prk: "19ef24a32c717b167f33a91d6f648bdf96596776afdb6377ac434c1c293ccb04",
            okm: "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4\
                  b61a96c8",
        },
    ]
}
