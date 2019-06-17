---
id: crypto
title: NextGen Crypto
custom_edit_url: https://github.com/libra/libra/edit/master/crypto/nextgen_crypto/README.md
---
# NextGen

The nextgen folder hosts the future version of the Libra crypto API and several algorithms that will be used in the upcoming versions.

## Overview

Nextgen contains the following implementations:

* traits.rs introduces new abstractions for the crypto API.
* Ed25519 performs signatures using the new API design based on [ed25519-dalek](https://docs.rs/ed25519-dalek/1.0.0-pre.1/ed25519_dalek/) library with additional security checks (e.g. for malleability).
* BLS12381 performs signatures using the new API design based on [threshold_crypto](https://github.com/poanetwork/threshold_crypto) library. BLS signatures currently undergo a [standartization process](https://tools.ietf.org/html/draft-boneh-bls-signature-00).
* ECVRF implements a verifiable random function (VRF) according to [draft-irtf-cfrg-vrf-04](https://tools.ietf.org/html/draft-irtf-cfrg-vrf-04) over curve25519.
* SLIP-0010 implements universal hierarchical key derivation for Ed25519 according to [SLIP-0010](https://github.com/satoshilabs/slips/blob/master/slip-0010.md).

## How is this module organized?
    nextgen_crypto/src
    ├── bls12381.rs         # Bls12-381 implementation of the signing/verification API in traits.rs
    ├── ed25519.rs          # Ed25519 implementation of the signing/verification API in traits.rs
    ├── lib.rs
    ├── slip0010.rs         # SLIP-0010 universal hierarchical key derivation for Ed25519
    ├── test_utils.rs
    ├── traits.rs           # New API design and the necessary abstractions
    ├── unit_tests/         # Tests
    └── vrf/                
        ├── ecvrf.rs        # ECVRF implementation using curve25519 and SHA512
        ├── mod.rs
        └── unit_tests      # Tests

