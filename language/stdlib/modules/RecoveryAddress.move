address 0x1 {
module RecoveryAddress {
    use 0x1::Errors;
    use 0x1::LibraAccount::{Self, KeyRotationCapability};
    use 0x1::Signer;
    use 0x1::VASP;
    use 0x1::Vector;

    /// A resource that holds the `KeyRotationCapability`s for several accounts belonging to the
    /// same VASP. A VASP account that delegates its `KeyRotationCapability` to
    /// but also allows the account that stores the `RecoveryAddress` resource to rotate its
    /// authentication key.
    /// This is useful as an account recovery mechanism: VASP accounts can all delegate their
    /// rotation capabilities to a single `RecoveryAddress` resource stored under address A.
    /// The authentication key for A can be "buried in the mountain" and dug up only if the need to
    /// recover one of accounts in `rotation_caps` arises.
    resource struct RecoveryAddress {
        rotation_caps: vector<KeyRotationCapability>
    }

    const ENOT_A_VASP: u64 = 0;
    const EKEY_ROTATION_DEPENDENCY_CYCLE: u64 = 1;
    const ECANNOT_ROTATE_KEY: u64 = 2;
    const EINVALID_KEY_ROTATION_DELEGATION: u64 = 3;
    const EACCOUNT_NOT_RECOVERABLE: u64 = 4;
    const ERECOVERY_ADDRESS: u64 = 5;

    /// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
    /// `RecoveryAddress` resource under  `recovery_account`.
    /// Aborts if `recovery_account` has delegated its `KeyRotationCapability`, already has a
    /// `RecoveryAddress` resource, or is not a VASP.
    public fun publish(recovery_account: &signer, rotation_cap: KeyRotationCapability) {
        let addr = Signer::address_of(recovery_account);
        // Only VASPs can create a recovery address
        assert(VASP::is_vasp(addr), Errors::invalid_argument(ENOT_A_VASP));
        // put the rotation capability for the recovery account itself in `rotation_caps`. This
        // ensures two things:
        // (1) It's not possible to get into a "recovery cycle" where A is the recovery account for
        //     B and B is the recovery account for A
        // (2) rotation_caps is always nonempty
        assert(
            *LibraAccount::key_rotation_capability_address(&rotation_cap) == addr,
             Errors::invalid_argument(EKEY_ROTATION_DEPENDENCY_CYCLE)
        );
        assert(!exists<RecoveryAddress>(addr), Errors::already_published(ERECOVERY_ADDRESS));
        move_to(
            recovery_account,
            RecoveryAddress { rotation_caps: Vector::singleton(rotation_cap) }
        )
    }
    spec fun publish {
        let addr = Signer::spec_address_of(recovery_account);
        aborts_if !VASP::is_vasp(addr) with Errors::INVALID_ARGUMENT;
        aborts_if spec_is_recovery_address(addr) with Errors::ALREADY_PUBLISHED;
        aborts_if LibraAccount::key_rotation_capability_address(rotation_cap) != addr
            with Errors::INVALID_ARGUMENT;
        ensures spec_is_recovery_address(Signer::spec_address_of(recovery_account));
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
        // Check that `recovery_address` has a `RecoveryAddress` resource
        assert(exists<RecoveryAddress>(recovery_address), Errors::not_published(ERECOVERY_ADDRESS));
        let sender = Signer::address_of(account);
        assert(
            // The original owner of a key rotation capability can rotate its own key
            sender == to_recover ||
            // The owner of the `RecoveryAddress` resource can rotate any key
            sender == recovery_address,
            Errors::invalid_argument(ECANNOT_ROTATE_KEY)
        );

        let caps = &borrow_global<RecoveryAddress>(recovery_address).rotation_caps;
        let i = 0;
        let len = Vector::length(caps);
        while ({
            spec {
                assert i <= len;
                assert forall j in 0..i: caps[j].account_address != to_recover;
            };
            (i < len)
        })
        {
            let cap = Vector::borrow(caps, i);
            if (LibraAccount::key_rotation_capability_address(cap) == &to_recover) {
                LibraAccount::rotate_authentication_key(cap, new_key);
                return
            };
            i = i + 1
        };
        spec {
            assert i == len;
            assert forall j in 0..len: caps[j].account_address != to_recover;
        };
        // Couldn't find `to_recover` in the account recovery resource; abort
        abort Errors::invalid_argument(EACCOUNT_NOT_RECOVERABLE)
    }
    spec fun rotate_authentication_key {
        pragma verify_duration_estimate = 100; // TODO: occasional timeout
        aborts_if !spec_is_recovery_address(recovery_address) with Errors::NOT_PUBLISHED;
        aborts_if !exists<LibraAccount::LibraAccount>(to_recover) with Errors::NOT_PUBLISHED;
        aborts_if len(new_key) != 32;
        aborts_if !spec_holds_key_rotation_cap_for(recovery_address, to_recover);
        aborts_if !(Signer::spec_address_of(account) == recovery_address
                    || Signer::spec_address_of(account) == to_recover);
        ensures global<LibraAccount::LibraAccount>(to_recover).authentication_key == new_key;
    }


    /// Add `to_recover` to the `RecoveryAddress` resource under `recovery_address`.
    /// Aborts if `to_recover.address` and `recovery_address belong to different VASPs, or if
    /// `recovery_address` does not have a `RecoveryAddress` resource.
    public fun add_rotation_capability(to_recover: KeyRotationCapability, recovery_address: address)
    acquires RecoveryAddress {
        // Check that `recovery_address` has a `RecoveryAddress` resource
        assert(exists<RecoveryAddress>(recovery_address), Errors::not_published(ERECOVERY_ADDRESS));
        // Only accept the rotation capability if both accounts belong to the same VASP
        let to_recover_address = *LibraAccount::key_rotation_capability_address(&to_recover);
        assert(
            VASP::parent_address(recovery_address) == VASP::parent_address(to_recover_address),
            Errors::invalid_argument(EINVALID_KEY_ROTATION_DELEGATION)
        );

        Vector::push_back(
            &mut borrow_global_mut<RecoveryAddress>(recovery_address).rotation_caps,
            to_recover
        );
    }
    spec fun add_rotation_capability {
        pragma verify = false; // TODO: timeout
    }

    // ****************** SPECIFICATIONS *******************

    /// # Module specifications

    spec module {
        pragma verify = true;
    }

    spec module {
        /// Returns true if `addr` is a recovery address.
        define spec_is_recovery_address(addr: address): bool
        {
            exists<RecoveryAddress>(addr)
        }

        /// Returns all the `KeyRotationCapability`s held at `recovery_address`.
        define spec_get_rotation_caps(recovery_address: address):
            vector<LibraAccount::KeyRotationCapability>
        {
            global<RecoveryAddress>(recovery_address).rotation_caps
        }

        /// Returns true if `recovery_address` holds the
        /// `KeyRotationCapability` for `addr`.
        define spec_holds_key_rotation_cap_for(
            recovery_address: address,
            addr: address): bool
        {
            // BUG: the commented out version will break the postconditions.
            // exists i in 0..len(spec_get_rotation_caps(recovery_address)):
            //     spec_get_rotation_caps(recovery_address)[i].account_address == addr
            exists i: u64
                where 0 <= i && i < len(spec_get_rotation_caps(recovery_address)):
                    spec_get_rotation_caps(recovery_address)[i].account_address == addr
        }
    }

    /// ## RecoveryAddress has its own KeyRotationCapability

    spec module {
        // TODO: add_rotation_capability does not seem to respect this invariant. Is the invariant wrong or
        // the function?
        /*
        invariant [global, on_update]
            forall addr1: address where spec_is_recovery_address(addr1):
                len(spec_get_rotation_caps(addr1)) > 0 &&
                spec_get_rotation_caps(addr1)[0].account_address == addr1;
        */
    }

    /// ## RecoveryAddress resource stays

    spec module {
        invariant update [global]
           forall addr1: address:
               old(spec_is_recovery_address(addr1)) ==> spec_is_recovery_address(addr1);
    }

    /// ## RecoveryAddress remains same

    spec module {
        invariant update [global]
            forall recovery_addr: address, to_recovery_addr: address
            where old(spec_is_recovery_address(recovery_addr)):
                old(spec_holds_key_rotation_cap_for(recovery_addr, to_recovery_addr))
                ==> spec_holds_key_rotation_cap_for(recovery_addr, to_recovery_addr);
    }


    /// ## Only VASPs can be RecoveryAddress

    spec module {
        invariant [global, on_update]
            forall recovery_addr: address where spec_is_recovery_address(recovery_addr):
                VASP::is_vasp(recovery_addr);
    }

    /// # Specifications for individual functions

     spec fun add_rotation_capability {
        aborts_if !spec_is_recovery_address(recovery_address);
        aborts_if VASP::spec_parent_address(recovery_address) != VASP::spec_parent_address(LibraAccount::key_rotation_capability_address(to_recover));
        ensures spec_get_rotation_caps(recovery_address)[
            len(spec_get_rotation_caps(recovery_address)) - 1] == to_recover;
    }
}
}
