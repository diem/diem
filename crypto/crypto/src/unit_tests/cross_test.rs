// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// This is necessary for the derive macros which rely on being used in a
// context where the crypto crate is external
use crate as diem_crypto;
use crate::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey, MultiEd25519Signature},
    test_utils::{random_serializable_struct, uniform_keypair_strategy},
    traits::*,
};

use diem_crypto_derive::{
    PrivateKey, PublicKey, Signature, SigningKey, SilentDebug, ValidCryptoMaterial, VerifyingKey,
};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

// Here we aim to make a point about how we can build an enum generically
// on top of a few choice signing scheme types. This enum implements the
// VerifyingKey, SigningKey for precisely the types selected for that enum
// (here, Ed25519(PublicKey|PrivateKey|Signature) and MultiEd25519(...)).
//
// Note that we do not break type safety (see towards the end), and that
// this enum can safely be put into the usual collection (Vec, HashMap).
//

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    ValidCryptoMaterial,
    PublicKey,
    VerifyingKey,
)]
#[PrivateKeyType = "PrivateK"]
#[SignatureType = "Sig"]
enum PublicK {
    Ed(Ed25519PublicKey),
    MultiEd(MultiEd25519PublicKey),
}

#[derive(Serialize, Deserialize, SilentDebug, ValidCryptoMaterial, PrivateKey, SigningKey)]
#[PublicKeyType = "PublicK"]
#[SignatureType = "Sig"]
enum PrivateK {
    Ed(Ed25519PrivateKey),
    MultiEd(MultiEd25519PrivateKey),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Signature)]
#[PublicKeyType = "PublicK"]
#[PrivateKeyType = "PrivateK"]
enum Sig {
    Ed(Ed25519Signature),
    MultiEd(MultiEd25519Signature),
}

///////////////////////////////////////////////////////
// End of declarations — let's now prove type safety //
///////////////////////////////////////////////////////
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    #[allow(deprecated)]
    fn test_keys_mix(
        message in random_serializable_struct(),
        ed_keypair1 in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>(),
        ed_keypair2 in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>(),
        med_keypair in uniform_keypair_strategy::<MultiEd25519PrivateKey, MultiEd25519PublicKey>()
    ) {
        // this is impossible to write statically, due to the trait not being
        // object-safe (voluntarily), see unsupported_sigs below
        // let mut l: Vec<Box<dyn PrivateKey>> = vec![];
        let mut l: Vec<Ed25519PrivateKey> = vec![ed_keypair1.private_key];
        let ed_key = l.pop().unwrap();
        let signature = ed_key.sign(&message);

        // This is business as usual
        prop_assert!(signature.verify(&message, &ed_keypair1.public_key).is_ok());

        // signature-publickey mixups are statically impossible, see unsupported_sigs below
        let mut l2: Vec<PrivateK> = vec![
            PrivateK::MultiEd(med_keypair.private_key),
            PrivateK::Ed(ed_keypair2.private_key),
        ];

        let ed_key = l2.pop().unwrap();
        let ed_signature = ed_key.sign(&message);

        // This is still business as usual
        let ed_pubkey2 = PublicK::Ed(ed_keypair2.public_key);
        let good_sigver = ed_signature.verify(&message, &ed_pubkey2);
        prop_assert!(good_sigver.is_ok(), "{:?}", good_sigver);

        // but this still fails, as expected
        let med_pubkey = PublicK::MultiEd(med_keypair.public_key);
        let bad_sigver = ed_signature.verify(&message, &med_pubkey);
        prop_assert!(bad_sigver.is_err(), "{:?}", bad_sigver);

        // And now just in case we're confused again, we pop in the
        // reverse direction
        let med_key = l2.pop().unwrap();
        let med_signature = med_key.sign(&message);

        // This is still business as usual
        let good_sigver = med_signature.verify(&message, &med_pubkey);
        prop_assert!(good_sigver.is_ok(), "{:?}", good_sigver);

        // but this still fails, as expected
        let bad_sigver = med_signature.verify(&message, &ed_pubkey2);
        prop_assert!(bad_sigver.is_err(), "{:?}", bad_sigver);
    }
}

#[test]
fn unsupported_sigs() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/unit_tests/compilation/cross_test.rs");
    t.compile_fail("src/unit_tests/compilation/cross_test_trait_obj.rs");
    t.compile_fail("src/unit_tests/compilation/cross_test_trait_obj_sig.rs");
    t.compile_fail("src/unit_tests/compilation/cross_test_trait_obj_pub.rs");
}
