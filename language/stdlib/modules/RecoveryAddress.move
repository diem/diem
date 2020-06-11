address 0x1 {
module RecoveryAddress {
    use 0x1::LibraAccount;
    use 0x1::Signer;
    use 0x1::VASP;
    use 0x1::Vector;

    /// A resource that holds the `KeyRotationCapability`s for several accounts belonging to the
    /// same VASP. A VASP account that delegates its `KeyRotationCapability` to
    /// a `RecoveryAddress` resource retains the ability to rotate its own authentication key,
    /// but also allows the account that stores the `RecoveryAddress` resource to rotate its
    /// authentication key.
    /// This is useful as an account recovery mechanism: VASP accounts can all delegate their
    /// rotation capabilities to a single `RecoveryAddress` resource stored under address A.
    /// The authentication key for A can be "buried in the mountain" and dug up only if the need to
    /// recover one of accounts in `rotation_caps` arises.
    resource struct RecoveryAddress {
        rotation_caps: vector<LibraAccount::KeyRotationCapability>
    }

    /// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
    /// `RecoveryAddress` resource under  `recovery_account`.
    /// Aborts if `recovery_account` has delegated its `KeyRotationCapability`, already has a
    /// `RecoveryAddress` resource, or is not a VASP.
    public fun publish(recovery_account: &signer) {
        // Only VASPs can create a recovery address
        // TODO: proper error code
        assert(VASP::is_vasp(Signer::address_of(recovery_account)), 2222);
        // put the rotation capability for the recovery account itself in `rotation_caps`. This
        // ensures two things:
        // (1) It's not possible to get into a "recovery cycle" where A is the recovery account for
        //     B and B is the recovery account for A
        // (2) rotation_caps is always nonempty
        let rotation_cap = LibraAccount::extract_key_rotation_capability(recovery_account);
        move_to(
            recovery_account,
            RecoveryAddress { rotation_caps: Vector::singleton(rotation_cap) }
        )
    }

    /// Rotate the authentication key of `to_recover` to `new_key`. Can be invoked by either
    /// `recovery_address` or `to_recover`.
    /// Aborts if `recovery_address` does not have the `KeyRotationCapability` for `to_recover`.
    public fun rotate_authentication_key(
        account: &signer,
        recovery_address: address,
        to_recover: address,
        new_key: vector<u8>
    ) acquires RecoveryAddress {
        let sender = Signer::address_of(account);
        // Both the original owner `to_recover` of the KeyRotationCapability and the
        // `recovery_address` can rotate the authentication key
        // TODO: proper error code
        assert(sender == recovery_address || sender == to_recover, 3333);

        let caps = &borrow_global<RecoveryAddress>(recovery_address).rotation_caps;
        let i = 0;
        let len = Vector::length(caps);
        while (i < len) {
            let cap = Vector::borrow(caps, i);
            if (LibraAccount::key_rotation_capability_address(cap) == &to_recover) {
                LibraAccount::rotate_authentication_key(cap, new_key);
                return
            };
            i = i + 1
        };
        // Couldn't find `to_recover` in the account recovery resource; abort
        // TODO: proper error code
        abort(555)
    }

    /// Add the `KeyRotationCapability` for `to_recover_account` to the `RecoveryAddress`
    /// resource under `recovery_address`.
    /// Aborts if `to_recovery_account` and `to_recovery_address belong to different VASPs, if
    /// `recovery_address` does not have a `RecoveryAddress` resource, or if
    /// `to_recover_account` has already extracted its `KeyRotationCapability`.
    public fun add_rotation_capability(to_recover_account: &signer, recovery_address: address)
    acquires RecoveryAddress {
        let addr = Signer::address_of(to_recover_account);
        // Only accept the rotation capability if both accounts belong to the same VASP
        assert(
            VASP::parent_address(recovery_address) ==
                VASP::parent_address(addr),
            444 // TODO: proper error code
        );

        let caps = &mut borrow_global_mut<RecoveryAddress>(recovery_address).rotation_caps;
        Vector::push_back(caps, LibraAccount::extract_key_rotation_capability(to_recover_account));
    }
}
}
