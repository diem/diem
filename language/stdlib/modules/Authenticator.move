// Move representation of the authenticator types used in Libra:
// - Ed25519 (single-sig)
// - MultiEd25519 (K-of-N multisig)

address 0x1 {
module Authenticator {
    use 0x1::Errors;
    use 0x1::Hash;
    use 0x1::LCS;
    use 0x1::Vector;

    // A multi-ed25519 public key
    struct MultiEd25519PublicKey {
        // vector of ed25519 public keys
        public_keys: vector<vector<u8>>,
        // approval threshold
        threshold: u8,
    }

    const EZERO_THRESHOLD: u64 = 0;
    const ENOT_ENOUGH_KEYS_FOR_THRESHOLD: u64 = 1;
    const ENUM_KEYS_ABOVE_MAX_THRESHOLD: u64 = 2;

    // Create a a multisig policy from a vector of ed25519 public keys and a threshold.
    // Note: this does *not* check uniqueness of keys. Repeated keys are convenient to
    // encode weighted multisig policies. For example Alice AND 1 of Bob or Carol is
    // public_key: {alice_key, alice_key, bob_key, carol_key}, threshold: 3
    // Aborts if threshold is zero or bigger than the length of `public_keys`.
    public fun create_multi_ed25519(
        public_keys: vector<vector<u8>>,
        threshold: u8
    ): MultiEd25519PublicKey {
        // check theshold requirements
        let len = Vector::length(&public_keys);
        assert(threshold != 0, Errors::invalid_argument(EZERO_THRESHOLD));
        assert(
            (threshold as u64) <= len,
            Errors::invalid_argument(ENOT_ENOUGH_KEYS_FOR_THRESHOLD)
        );
        // TODO: add constant MULTI_ED25519_MAX_KEYS
        // the multied25519 signature scheme allows at most 32 keys
        assert(len <= 32, Errors::invalid_argument(ENUM_KEYS_ABOVE_MAX_THRESHOLD));

        MultiEd25519PublicKey { public_keys, threshold }
    }

    // Compute an authentication key for the ed25519 public key `public_key`
    public fun ed25519_authentication_key(public_key: vector<u8>): vector<u8> {
        // TODO: add constant ED25519_SCHEME_ID = 0u8
        Vector::push_back(&mut public_key, 0u8);
        Hash::sha3_256(public_key)
    }

    // Compute a multied25519 account authentication key for the policy `k`
    public fun multi_ed25519_authentication_key(k: &MultiEd25519PublicKey): vector<u8> {
        let public_keys = &k.public_keys;
        let len = Vector::length(public_keys);
        let authentication_key_preimage = Vector::empty();
        let i = 0;
        while (i < len) {
            let public_key = *Vector::borrow(public_keys, i);
            Vector::append(
                &mut authentication_key_preimage,
                public_key
            );
            i = i + 1;
        };
        Vector::append(&mut authentication_key_preimage, LCS::to_bytes(&k.threshold));
        // TODO: add constant MULTI_ED25519_SCHEME_ID = 1u8
        Vector::push_back(&mut authentication_key_preimage, 1u8);
        Hash::sha3_256(authentication_key_preimage)
    }

    // Return the public keys involved in the multisig policy `k`
    public fun public_keys(k: &MultiEd25519PublicKey): &vector<vector<u8>> {
        &k.public_keys
    }

    // Return the threshold for the multisig policy `k`
    public fun threshold(k: &MultiEd25519PublicKey): u8 {
        *&k.threshold
    }

}
}
