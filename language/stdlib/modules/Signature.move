address 0x1 {

/// Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.
module Signature {
    native public fun ed25519_validate_pubkey(public_key: vector<u8>): bool;
    native public fun ed25519_verify(signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): bool;
    native public fun ed25519_threshold_verify(bitmap: vector<u8>, signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): u64;

    /// The signature verification functions are treated as uninterpreted. The uninterpreted semantics of
    /// the Move functions aligns with that of the specification functions below.
    spec module {
        native define spec_ed25519_validate_pubkey(public_key: vector<u8>): bool;
        native define spec_ed25519_verify(signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): bool;
        native define spec_ed25519_threshold_verify(bitmap: vector<u8>, signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): u64;
    }
}

}
