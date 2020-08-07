address 0x1 {

/// Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.
module Signature {
    native public fun ed25519_validate_pubkey(public_key: vector<u8>): bool;
    native public fun ed25519_verify(signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): bool;
}

}
