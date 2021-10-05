/// This module defines an account recovery mechanism that can be used by VASPs.
module DiemFramework::RecoveryAddress {
    use DiemFramework::DiemAccount::{Self, KeyRotationCapability};
    use DiemFramework::VASP;
    use Std::Errors;
    use Std::Signer;
    use Std::Vector;

    /// A resource that holds the `KeyRotationCapability`s for several accounts belonging to the
    /// same VASP. A VASP account that delegates its `KeyRotationCapability` to
    /// but also allows the account that stores the `RecoveryAddress` resource to rotate its
    /// authentication key.
    /// This is useful as an account recovery mechanism: VASP accounts can all delegate their
    /// rotation capabilities to a single `RecoveryAddress` resource stored under address A.
    /// The authentication key for A can be "buried in the mountain" and dug up only if the need to
    /// recover one of accounts in `rotation_caps` arises.
    struct RecoveryAddress has key {
        rotation_caps: vector<KeyRotationCapability>
    }

    /// Only VASPs can create a recovery address
    const ENOT_A_VASP: u64 = 0;
    /// A cycle would have been created would be created
    const EKEY_ROTATION_DEPENDENCY_CYCLE: u64 = 1;
    /// The signer doesn't have the appropriate privileges to rotate the account's key
    const ECANNOT_ROTATE_KEY: u64 = 2;
    /// Only accounts belonging to the same VASP can delegate their key rotation capability
    const EINVALID_KEY_ROTATION_DELEGATION: u64 = 3;
    /// The account address couldn't be found in the account recovery resource
    const EACCOUNT_NOT_RECOVERABLE: u64 = 4;
    /// A `RecoveryAddress` resource was in an unexpected state
    const ERECOVERY_ADDRESS: u64 = 5;
    /// The maximum allowed number of keys have been registered with this recovery address.
    const EMAX_KEYS_REGISTERED: u64 = 6;

    /// The maximum number of keys that can be registered with a single recovery address.
    const MAX_REGISTERED_KEYS: u64 = 256;

    /// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
    /// `RecoveryAddress` resource under  `recovery_account`.
    /// Aborts if `recovery_account` has delegated its `KeyRotationCapability`, already has a
    /// `RecoveryAddress` resource, or is not a VASP.
    public fun publish(recovery_account: &signer, rotation_cap: KeyRotationCapability) {
        let addr = Signer::address_of(recovery_account);
        // Only VASPs can create a recovery address
        assert!(VASP::is_vasp(addr), Errors::invalid_argument(ENOT_A_VASP));
        // put the rotation capability for the recovery account itself in `rotation_caps`. This
        // ensures two things:
        // (1) It's not possible to get into a "recovery cycle" where A is the recovery account for
        //     B and B is the recovery account for A
        // (2) rotation_caps is always nonempty
        assert!(
            *DiemAccount::key_rotation_capability_address(&rotation_cap) == addr,
             Errors::invalid_argument(EKEY_ROTATION_DEPENDENCY_CYCLE)
        );
        assert!(!exists<RecoveryAddress>(addr), Errors::already_published(ERECOVERY_ADDRESS));
        move_to(
            recovery_account,
            RecoveryAddress { rotation_caps: Vector::singleton(rotation_cap) }
        )
    }
    spec publish {
        include PublishAbortsIf;
        include PublishEnsures;
    }
    spec schema PublishAbortsIf {
        recovery_account: signer;
        rotation_cap: KeyRotationCapability;
        let addr = Signer::address_of(recovery_account);
        aborts_if !VASP::is_vasp(addr) with Errors::INVALID_ARGUMENT;
        aborts_if spec_is_recovery_address(addr) with Errors::ALREADY_PUBLISHED;
        aborts_if DiemAccount::key_rotation_capability_address(rotation_cap) != addr
            with Errors::INVALID_ARGUMENT;
    }
    spec schema PublishEnsures {
        recovery_account: signer;
        rotation_cap: KeyRotationCapability;
        let addr = Signer::address_of(recovery_account);
        ensures spec_is_recovery_address(addr);
        ensures len(spec_get_rotation_caps(addr)) == 1;
        ensures spec_get_rotation_caps(addr)[0] == rotation_cap;
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
        assert!(exists<RecoveryAddress>(recovery_address), Errors::not_published(ERECOVERY_ADDRESS));
        let sender = Signer::address_of(account);
        assert!(
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
                invariant i <= len;
                invariant forall j in 0..i: caps[j].account_address != to_recover;
            };
            (i < len)
        })
        {
            let cap = Vector::borrow(caps, i);
            if (DiemAccount::key_rotation_capability_address(cap) == &to_recover) {
                DiemAccount::rotate_authentication_key(cap, new_key);
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
    spec rotate_authentication_key {
        include RotateAuthenticationKeyAbortsIf;
        include RotateAuthenticationKeyEnsures;
    }
    spec schema RotateAuthenticationKeyAbortsIf {
        account: signer;
        recovery_address: address;
        to_recover: address;
        new_key: vector<u8>;
        aborts_if !spec_is_recovery_address(recovery_address) with Errors::NOT_PUBLISHED;
        aborts_if !DiemAccount::exists_at(to_recover) with Errors::NOT_PUBLISHED;
        aborts_if len(new_key) != 32 with Errors::INVALID_ARGUMENT;
        aborts_if !spec_holds_key_rotation_cap_for(recovery_address, to_recover) with Errors::INVALID_ARGUMENT;
        aborts_if !(Signer::address_of(account) == recovery_address
                    || Signer::address_of(account) == to_recover) with Errors::INVALID_ARGUMENT;
    }
    spec schema RotateAuthenticationKeyEnsures {
        to_recover: address;
        new_key: vector<u8>;
        ensures DiemAccount::authentication_key(to_recover) == new_key;
    }

    /// Add `to_recover` to the `RecoveryAddress` resource under `recovery_address`.
    /// Aborts if `to_recover.address` and `recovery_address` belong to different VASPs, or if
    /// `recovery_address` does not have a `RecoveryAddress` resource.
    public fun add_rotation_capability(to_recover: KeyRotationCapability, recovery_address: address)
    acquires RecoveryAddress {
        // Check that `recovery_address` has a `RecoveryAddress` resource
        assert!(exists<RecoveryAddress>(recovery_address), Errors::not_published(ERECOVERY_ADDRESS));
        // Only accept the rotation capability if both accounts belong to the same VASP
        let to_recover_address = *DiemAccount::key_rotation_capability_address(&to_recover);
        assert!(
            VASP::is_same_vasp(recovery_address, to_recover_address),
            Errors::invalid_argument(EINVALID_KEY_ROTATION_DELEGATION)
        );

        let recovery_caps = &mut borrow_global_mut<RecoveryAddress>(recovery_address).rotation_caps;
        assert!(
            Vector::length(recovery_caps) < MAX_REGISTERED_KEYS,
            Errors::limit_exceeded(EMAX_KEYS_REGISTERED)
        );

        Vector::push_back(recovery_caps, to_recover);
    }
    spec add_rotation_capability {
        include AddRotationCapabilityAbortsIf;
        include AddRotationCapabilityEnsures;
    }
    spec schema AddRotationCapabilityAbortsIf {
        to_recover: KeyRotationCapability;
        recovery_address: address;
        aborts_if !spec_is_recovery_address(recovery_address) with Errors::NOT_PUBLISHED;
        aborts_if len(global<RecoveryAddress>(recovery_address).rotation_caps) >= MAX_REGISTERED_KEYS with Errors::LIMIT_EXCEEDED;
        let to_recover_address = DiemAccount::key_rotation_capability_address(to_recover);
        aborts_if !VASP::spec_is_same_vasp(recovery_address, to_recover_address) with Errors::INVALID_ARGUMENT;
    }
    spec schema AddRotationCapabilityEnsures {
        to_recover: KeyRotationCapability;
        recovery_address: address;
        let post num_rotation_caps = len(spec_get_rotation_caps(recovery_address));
        ensures spec_get_rotation_caps(recovery_address)[num_rotation_caps - 1] == to_recover;
    }

    // ****************** SPECIFICATIONS *******************
    spec module {} // switch documentation context back to module level


    /// # Initialization

    spec module {
        /// A RecoveryAddress has its own `KeyRotationCapability`.
        invariant forall addr: address
            where spec_is_recovery_address(addr): (
                len(spec_get_rotation_caps(addr)) > 0 &&
                spec_get_rotation_caps(addr)[0].account_address == addr
            );
    }

    /// # Persistence of Resource

    spec module {
        invariant update
           forall addr: address:
               old(spec_is_recovery_address(addr)) ==> spec_is_recovery_address(addr);
    }

    /// # Persistence of KeyRotationCapability

    spec module {
        /// `RecoveryAddress` persists
        invariant update forall addr: address where old(exists<RecoveryAddress>(addr)):
            exists<RecoveryAddress>(addr);

        /// If `recovery_addr` holds the `KeyRotationCapability` of `to_recovery_addr`
        /// in the previous state, then it continues to hold the capability after the update.
        invariant update
            forall recovery_addr: address, to_recovery_addr: address
            where old(spec_is_recovery_address(recovery_addr)):
                old(spec_holds_key_rotation_cap_for(recovery_addr, to_recovery_addr))
                ==> spec_holds_key_rotation_cap_for(recovery_addr, to_recovery_addr);
    }


    /// # Consistency Between Resources and Roles

    spec module {
        /// Only VASPs can hold `RecoverAddress` resources.
        invariant forall addr: address
            where spec_is_recovery_address(addr): VASP::is_vasp(addr);
    }

    /// # Helper Functions

    spec module {
        /// Returns true if `addr` is a recovery address.
        fun spec_is_recovery_address(addr: address): bool
        {
            exists<RecoveryAddress>(addr)
        }

        /// Returns all the `KeyRotationCapability`s held at `recovery_address`.
        fun spec_get_rotation_caps(recovery_address: address):
            vector<DiemAccount::KeyRotationCapability>
        {
            global<RecoveryAddress>(recovery_address).rotation_caps
        }

        /// Returns true if `recovery_address` holds the
        /// `KeyRotationCapability` for `addr`.
        fun spec_holds_key_rotation_cap_for(
            recovery_address: address,
            addr: address): bool
        {
            exists i: u64
                where 0 <= i && i < len(spec_get_rotation_caps(recovery_address)):
                    spec_get_rotation_caps(recovery_address)[i].account_address == addr
        }
    }

}
