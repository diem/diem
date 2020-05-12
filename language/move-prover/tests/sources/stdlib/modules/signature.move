address 0x0 {

module Signature {
    // Validation of an EdDSA public key, including invalid point & small subgroup checks
    native public fun ed25519_validate_pubkey(public_key: vector<u8>): bool;
    // EdDSA Signature Verification
    native public fun ed25519_verify(signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): bool;
    // k-out-of-n EdDSA Signature Verification, where the subset of signers are marked by a bitmap
    native public fun ed25519_threshold_verify(bitmap: vector<u8>, signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): u64;
}
}
