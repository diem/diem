// Each address that holds a `SharedEd25519PublicKey` resource can rotate the public key stored in
// this resource, but the account's authentication key will be updated in lockstep. This ensures
// that the two keys always stay in sync.

address 0x1 {
module SharedEd25519PublicKey {
    use 0x1::Authenticator;
    use 0x1::Errors;
    use 0x1::LibraAccount;
    use 0x1::Signature;
    use 0x1::Signer;

    // A resource that forces the account associated with `rotation_cap` to use a ed25519
    // authentication key derived from `key`
    resource struct SharedEd25519PublicKey {
        // 32 byte ed25519 public key
        key: vector<u8>,
        // rotation capability for an account whose authentication key is always derived from `key`
        rotation_cap: LibraAccount::KeyRotationCapability,
    }

    const EMALFORMED_PUBLIC_KEY: u64 = 0;
    const ESHARED_KEY: u64 = 1;

    // (1) Rotate the authentication key of the sender to `key`
    // (2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability
    //     of the sender under the `account`'s address.
    // Aborts if the sender already has a `SharedEd25519PublicKey` resource.
    // Aborts if the length of `new_public_key` is not 32.
    public fun publish(account: &signer, key: vector<u8>) {
        let t = SharedEd25519PublicKey {
            key: x"",
            rotation_cap: LibraAccount::extract_key_rotation_capability(account)
        };
        rotate_key_(&mut t, key);
        assert(!exists<SharedEd25519PublicKey>(Signer::address_of(account)), Errors::already_published(ESHARED_KEY));
        move_to(account, t);
    }

    fun rotate_key_(shared_key: &mut SharedEd25519PublicKey, new_public_key: vector<u8>) {
        // Cryptographic check of public key validity
        assert(
            Signature::ed25519_validate_pubkey(copy new_public_key),
            Errors::invalid_argument(EMALFORMED_PUBLIC_KEY)
        );
        LibraAccount::rotate_authentication_key(
            &shared_key.rotation_cap,
            Authenticator::ed25519_authentication_key(copy new_public_key)
        );
        shared_key.key = new_public_key;
    }

    // (1) rotate the public key stored `account`'s `SharedEd25519PublicKey` resource to
    // `new_public_key`
    // (2) rotate the authentication key using the capability stored in the `account`'s
    // `SharedEd25519PublicKey` to a new value derived from `new_public_key`
    // Aborts if the sender does not have a `SharedEd25519PublicKey` resource.
    // Aborts if the length of `new_public_key` is not 32.
    public fun rotate_key(account: &signer, new_public_key: vector<u8>) acquires SharedEd25519PublicKey {
        rotate_key_(borrow_global_mut<SharedEd25519PublicKey>(Signer::address_of(account)), new_public_key);
    }

    // Return the public key stored under `addr`.
    // Aborts if `addr` does not hold a `SharedEd25519PublicKey` resource.
    public fun key(addr: address): vector<u8> acquires SharedEd25519PublicKey {
        assert(exists<SharedEd25519PublicKey>(addr), Errors::not_published(ESHARED_KEY));
        *&borrow_global<SharedEd25519PublicKey>(addr).key
    }

    // Returns true if `addr` holds a `SharedEd25519PublicKey` resource.
    public fun exists_at(addr: address): bool {
        exists<SharedEd25519PublicKey>(addr)
    }

}
}
