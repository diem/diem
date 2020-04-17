// Each address that holds a `SharedEd25519PublicKey` resource can rotate the public key stored in
// this resource, but the account's authentication key will be updated in lockstep. This ensures
// that the two keys always stay in sync.

address 0x0:
module SharedEd25519PublicKey {
    use 0x0::Authenticator;
    use 0x0::LibraAccount;
    use 0x0::Transaction;
    use 0x0::Vector;

    // A resource that forces the account associated with `rotation_cap` to use a ed25519
    // authentication key derived from `key`
    resource struct T {
        // 32 byte ed25519 public key
        key: vector<u8>,
        // rotation capability for an account whose authentication key is always derived from `key`
        rotation_cap: LibraAccount::KeyRotationCapability,
    }

    // (1) Rotate the authentication key of the sender to `key`
    // (2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability
    //     of the sender under the sender's address.
    // Aborts if the sender already has a SharedEd25519PublicKey resource.
    // Aborts if the length of `new_public_key` is not 32.
    public fun publish(key: vector<u8>) {
        let t = T {
            key: x"",
            rotation_cap: LibraAccount::extract_sender_key_rotation_capability()
        };
        rotate_key(&mut t, key);
        move_to_sender(t);
    }

    fun rotate_key(shared_key: &mut T, new_public_key: vector<u8>) {
        Transaction::assert(Vector::length(&new_public_key) == 32, 7000);
        let old_public_key = &mut shared_key.key;
        LibraAccount::rotate_authentication_key_with_capability(
            &shared_key.rotation_cap,
            Authenticator::ed25519_authentication_key(copy new_public_key)
        );
        *old_public_key = new_public_key;
    }

    // (1) rotate the public key stored in the sender's `SharedEd25519PublicKey` resource to
    // `new_public_key`
    // (2) rotate the authentication key using the capability stored in the sender's
    // `SharedEd25519PublicKey` to a new value derived from `new_public_key`
    // Aborts if the length of `new_public_key` is not 32.
    public fun rotate_sender_key(new_public_key: vector<u8>) acquires T {
        rotate_key(borrow_global_mut<T>(Transaction::sender()), new_public_key);
    }

    // Return the public key stored under `addr`.
    // Aborts if `addr` does not hold a `SharedEd25519PublicKey` resource.
    public fun key(addr: address): vector<u8> acquires T {
        *&borrow_global<T>(addr).key
    }

}
