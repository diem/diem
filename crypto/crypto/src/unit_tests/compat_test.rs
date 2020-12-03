// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compat;
use digest::Digest;
use proptest::{collection::vec, prelude::*};

// sanity check our compatibility layer by testing some basic SHA3-256 test vectors.
#[test]
fn check_basic_sha3_256_test_vectors() {
    let one_million_a = vec![b'a'; 1_000_000];

    let tests: [(&[u8], &[u8]); 4] = [
        (
            b"",
            b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
        ),
        (
            b"abc",
            b"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            b"41c0dba2a9d6240849100376a8235e2c82e1b9998a999e21db32dd97496d3376",
        ),
        (
            &one_million_a,
            b"5c8875ae474a3634ba4fd55ec85bffd661f32aca75c6d699d0cdcb6c115891c1",
        ),
    ];

    for (input, expected_output) in &tests {
        let expected_output = hex::decode(expected_output).unwrap();
        let output1 = compat::Sha3_256::digest(input);
        let output2 = sha3::Sha3_256::digest(input);
        assert_eq!(&expected_output, &output1.as_slice());
        assert_eq!(&expected_output, &output2.as_slice());
    }
}

proptest! {
    // check that RustCrypto SHA3-256 and our compatibility wrapper over tiny-keccak
    // SHA3-256 have the exact same behaviour for random inputs.
    #[test]
    fn sha3_256_compatibility_test(
        updates in vec(vec(any::<u8>(), 0..200), 0..10)
    ) {
        let mut hasher1 = compat::Sha3_256::default();
        let mut hasher2 = sha3::Sha3_256::default();

        for update in updates {
            hasher1.update(&update);
            hasher2.update(&update);

            let out1 = hasher1.clone().finalize();
            let out2 = hasher2.clone().finalize();
            assert_eq!(&out1, &out2);
        }
    }
}
