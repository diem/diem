// This file is created to verify the native function in the standard BCS module.

module VerifySignature {
    use 0x1::Signature;

    public fun verify_ed25519_validate_pubkey(public_key: vector<u8>): bool {
        Signature::ed25519_validate_pubkey(public_key)
    }
    spec fun verify_ed25519_validate_pubkey {
        ensures result == Signature::ed25519_validate_pubkey(public_key);
    }

    public fun verify_ed25519_verify(
        signature: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool {
        Signature::ed25519_verify(signature, public_key, message)
    }
    spec fun verify_ed25519_verify {
        ensures result == Signature::ed25519_verify(signature, public_key, message);
    }
}
