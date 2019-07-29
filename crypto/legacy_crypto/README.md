---
id: crypto
title: Crypto
custom_edit_url: https://github.com/libra/libra/edit/master/crypto/legacy_crypto/README.md
---
# Legacy Crypto

The crypto component hosts all the implementations of cryptographic primitives we use in Libra: hashing, signing, and key derivation/generation. The NextGen directory contains implementations of cryptographic primitives that will be used in the upcoming versions: new crypto API Enforcing type safety, verifiable random functions, BLS signatures.

## Overview

Libra makes use of several cryptographic algorithms:

* SHA-3 as the main hash function. It is standardized in [FIPS 202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf). It is based on the [sha3](https://docs.rs/sha3/) library.
* X25519 to perform key exchanges. It is used to secure communications between validators via the [Noise Protocol Framework](http://www.noiseprotocol.org/noise.html). It is based on the x25519-dalek library.
* Ed25519 to perform signatures. It is used both for consensus signatures and for transaction signatures. EdDSA is planned to be added to the next revision of FIPS 186 as mentioned in [NIST SP 800-133 Rev. 1](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-133r1-draft.pdf). It is based on the [ed25519-dalek](https://docs.rs/ed25519-dalek/1.0.0-pre.1/ed25519_dalek/) library with additional security checks (e.g., for malleability).
* HKDF: HMAC-based Extract-and-Expand Key Derivation Function (HKDF) based on [RFC 5869](https://tools.ietf.org/html/rfc5869). It is used to generate keys from a salt (optional), seed, and application-info (optional).

## How is this module organized?
```
    legacy_crypto/src
    ├── signing.rs          # Ed25519 signature scheme
    ├── hash.rs             # Hash function (SHA-3)
    ├── hkdf.rs             # HKDF implementation (HMAC-based Extract-and-Expand Key Derivation Function based on RFC 5869)
    ├── x25519.rs           # X25519 keys generation
    ├── macros/             # Derivations for SilentDebug and SilentDisplay
    ├── utils.rs            # Serialization utility functions
    ├── unit_tests          # Tests
    └── lib.rs
```

Currently `x25519.rs` only exposes the logic for managing keys. The relevant cryptographic primitives to the Noise Protocol Framework are under the [snow](https://docs.rs/snow/0.5.2/snow/) crate.

