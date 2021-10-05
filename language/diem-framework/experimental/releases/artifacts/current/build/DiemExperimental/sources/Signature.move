/// Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.
module DiemFramework::Signature {

    /// Return `true` if the bytes in `public_key` can be parsed as a valid Ed25519 public key.
    /// Returns `false` if `public_key` is not 32 bytes OR is 32 bytes, but does not pass
    /// points-on-curve or small subgroup checks. See the Rust `diem_crypto::Ed25519PublicKey` type
    /// for more details.
    /// Does not abort.
    native public fun ed25519_validate_pubkey(public_key: vector<u8>): bool;

    /// Return true if the Ed25519 `signature` on `message` verifies against the Ed25519 public key
    /// `public_key`.
    /// Returns `false` if:
    /// - `signature` is not 64 bytes
    /// - `public_key` is not 32 bytes
    /// - `public_key` does not pass points-on-curve or small subgroup checks,
    /// - `signature` and `public_key` are valid, but the signature on `message` does not verify.
    /// Does not abort.
    native public fun ed25519_verify(
        signature: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;
}
