/// Each address that holds a `SharedEd25519PublicKey` resource can rotate the public key stored in
/// this resource, but the account's authentication key will be updated in lockstep. This ensures
/// that the two keys always stay in sync.
module DiemFramework::SharedEd25519PublicKey {
    use DiemFramework::Authenticator;
    use DiemFramework::DiemAccount;
    use DiemFramework::Signature;
    use Std::Errors;
    use Std::Signer;

    /// A resource that forces the account associated with `rotation_cap` to use a ed25519
    /// authentication key derived from `key`
    struct SharedEd25519PublicKey has key {
        /// 32 byte ed25519 public key
        key: vector<u8>,
        /// rotation capability for an account whose authentication key is always derived from `key`
        rotation_cap: DiemAccount::KeyRotationCapability,
    }

    /// The shared ed25519 public key is not valid ed25519 public key
    const EMALFORMED_PUBLIC_KEY: u64 = 0;
    /// A shared ed25519 public key resource was not in the required state
    const ESHARED_KEY: u64 = 1;

    /// (1) Rotate the authentication key of the sender to `key`
    /// (2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability
    ///     of the sender under the `account`'s address.
    /// Aborts if the sender already has a `SharedEd25519PublicKey` resource.
    /// Aborts if the length of `new_public_key` is not 32.
    public fun publish(account: &signer, key: vector<u8>) {
        let t = SharedEd25519PublicKey {
            key: x"",
            rotation_cap: DiemAccount::extract_key_rotation_capability(account)
        };
        rotate_key_(&mut t, key);
        assert(!exists_at(Signer::address_of(account)), Errors::already_published(ESHARED_KEY));
        move_to(account, t);
    }
    spec publish {
        include PublishAbortsIf;
        include PublishEnsures;
    }
    spec schema PublishAbortsIf {
        account: signer;
        key: vector<u8>;
        let addr = Signer::address_of(account);
        include DiemAccount::ExtractKeyRotationCapabilityAbortsIf;
        include RotateKey_AbortsIf {
                shared_key: SharedEd25519PublicKey {
                    key: x"",
                    rotation_cap: DiemAccount::spec_get_key_rotation_cap(addr)
                },
                new_public_key: key
        };
        aborts_if exists_at(addr) with Errors::ALREADY_PUBLISHED;
    }
    spec schema PublishEnsures {
        account: signer;
        key: vector<u8>;
        let addr = Signer::address_of(account);

        ensures exists_at(addr);
        include RotateKey_Ensures { shared_key: global<SharedEd25519PublicKey>(addr), new_public_key: key};
    }

    fun rotate_key_(shared_key: &mut SharedEd25519PublicKey, new_public_key: vector<u8>) {
        // Cryptographic check of public key validity
        assert(
            Signature::ed25519_validate_pubkey(copy new_public_key),
            Errors::invalid_argument(EMALFORMED_PUBLIC_KEY)
        );
        DiemAccount::rotate_authentication_key(
            &shared_key.rotation_cap,
            Authenticator::ed25519_authentication_key(copy new_public_key)
        );
        shared_key.key = new_public_key;
    }
    spec rotate_key_ {
        include RotateKey_AbortsIf;
        include RotateKey_Ensures;
    }
    spec schema RotateKey_AbortsIf {
        shared_key: SharedEd25519PublicKey;
        new_public_key: vector<u8>;
        aborts_if !Signature::ed25519_validate_pubkey(new_public_key) with Errors::INVALID_ARGUMENT;
        include DiemAccount::RotateAuthenticationKeyAbortsIf {
            cap: shared_key.rotation_cap,
            new_authentication_key: Authenticator::spec_ed25519_authentication_key(new_public_key)
        };
    }
    spec schema RotateKey_Ensures {
        shared_key: SharedEd25519PublicKey;
        new_public_key: vector<u8>;
        ensures shared_key.key == new_public_key;
    }

    /// (1) rotate the public key stored `account`'s `SharedEd25519PublicKey` resource to
    /// `new_public_key`
    /// (2) rotate the authentication key using the capability stored in the `account`'s
    /// `SharedEd25519PublicKey` to a new value derived from `new_public_key`
    /// Aborts if the sender does not have a `SharedEd25519PublicKey` resource.
    /// Aborts if the length of `new_public_key` is not 32.
    public fun rotate_key(account: &signer, new_public_key: vector<u8>) acquires SharedEd25519PublicKey {
        let addr = Signer::address_of(account);
        assert(exists_at(addr), Errors::not_published(ESHARED_KEY));
        rotate_key_(borrow_global_mut<SharedEd25519PublicKey>(addr), new_public_key);
    }
    spec rotate_key {
        include RotateKeyAbortsIf;
        include RotateKeyEnsures;
    }
    spec schema RotateKeyAbortsIf {
        account: signer;
        new_public_key: vector<u8>;
        let addr = Signer::address_of(account);
        aborts_if !exists_at(addr) with Errors::NOT_PUBLISHED;
        include RotateKey_AbortsIf {shared_key: global<SharedEd25519PublicKey>(addr)};
    }
    spec schema RotateKeyEnsures {
        account: signer;
        new_public_key: vector<u8>;
        let addr = Signer::address_of(account);
        include RotateKey_Ensures {shared_key: global<SharedEd25519PublicKey>(addr)};
    }

    /// Return the public key stored under `addr`.
    /// Aborts if `addr` does not hold a `SharedEd25519PublicKey` resource.
    public fun key(addr: address): vector<u8> acquires SharedEd25519PublicKey {
        assert(exists_at(addr), Errors::not_published(ESHARED_KEY));
        *&borrow_global<SharedEd25519PublicKey>(addr).key
    }

    /// Returns true if `addr` holds a `SharedEd25519PublicKey` resource.
    public fun exists_at(addr: address): bool {
        exists<SharedEd25519PublicKey>(addr)
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Persistence
    spec module {
        invariant update forall addr: address where old(exists<SharedEd25519PublicKey>(addr)):
            exists<SharedEd25519PublicKey>(addr);
    }

}
