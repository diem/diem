address 0x1 {

/// This module keeps a global wall clock that stores the current Unix time in microseconds.
/// It interacts with the other modules in the following ways:
///
/// * Genesis: to initialize the timestamp
/// * VASP: to keep track of when credentials expire
/// * LibraSystem, LibraAccount, LibraConfig: to check if the current state is in the genesis state
/// * LibraBlock: to reach consensus on the global wall clock time
/// * AccountLimits: to limit the time of account limits
/// * LibraTransactionTimeout: to determine whether a transaction is still valid
///
module LibraTimestamp {
    use 0x1::CoreAddresses;
    use 0x1::Signer;

    /// A singleton resource holding the current Unix time in microseconds
    resource struct CurrentTimeMicroseconds {
        microseconds: u64,
    }

    /// A singleton resource used to determine whether time has started. This
    /// is called at the end of genesis.
    resource struct TimeHasStarted {}

    /// Initializes the global wall clock time resource. This can only be called from genesis.
    public fun initialize(association: &signer) {
        // Operational constraint, only callable by the Association address
        assert(Signer::address_of(association) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 1);

        // TODO: Should the initialized value be passed in to genesis?
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        move_to(association, timer);
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis.
    public fun set_time_has_started(association: &signer) acquires CurrentTimeMicroseconds {
        assert(Signer::address_of(association) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 1);

        // Current time must have been initialized.
        assert(
            exists<CurrentTimeMicroseconds>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()) && now_microseconds() == 0,
            2
        );
        move_to(association, TimeHasStarted{});
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        // Can only be invoked by LibraVM privilege.
        assert(Signer::address_of(account) == CoreAddresses::VM_RESERVED_ADDRESS(), 33);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        if (proposer == CoreAddresses::VM_RESERVED_ADDRESS()) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert(timestamp == global_timer.microseconds, 5001);
        } else {
            // Normal block. Time must advance
            assert(global_timer.microseconds < timestamp, 5001);
        };
        global_timer.microseconds = timestamp;
    }

    /// Gets the timestamp representing `now` in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        borrow_global<CurrentTimeMicroseconds>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()).microseconds
    }

    /// Helper function to determine if the blockchain is in genesis state.
    public fun is_genesis(): bool {
        !exists<TimeHasStarted>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS())
    }

    /// Helper function to determine whether the CurrentTime has been initialized.
    public fun is_not_initialized(): bool acquires CurrentTimeMicroseconds {
       !exists<CurrentTimeMicroseconds>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()) || now_microseconds() == 0
    }

    /// Helper function which aborts if not in genesis.
    public fun assert_is_genesis() {
        assert(is_genesis(), 0);
    }

    // **************** GLOBAL SPECIFICATION ****************

    /// # Module specification

    spec module {
        /// Verify all functions in this module.
        pragma verify = true;

        /// Specification version of the `Self::is_genesis` function.
        define spec_is_genesis(): bool {
            !exists<TimeHasStarted>(CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS())
        }

        /// True if the association root account has a CurrentTimeMicroseconds.
        define root_ctm_initialized(): bool {
            exists<CurrentTimeMicroseconds>(CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS())
        }

        /// Auxiliary function to get the association's Unix time in microseconds.
        define assoc_unix_time(): u64 {
            global<CurrentTimeMicroseconds>(CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS()).microseconds
        }
    }

    /// ## Persistence of Initialization

    spec schema InitializationPersists {
        /// If the `TimeHasStarted` resource is initialized and we finished genesis, we can never enter genesis again.
        /// Note that this is an important safety property since during genesis, we are allowed to perform certain
        /// operations which should never be allowed in normal on-chain execution.
        ensures old(!spec_is_genesis()) ==> !spec_is_genesis();

        /// If the `CurrentTimeMicroseconds` resource is initialized, it stays initialized.
        ensures old(root_ctm_initialized()) ==> root_ctm_initialized();
    }

    spec module {
        apply InitializationPersists to *;
    }

    /// ## Global Clock Time Progression

    spec schema GlobalWallClockIsMonotonic {
        /// The global wall clock time never decreases.
        ensures old(root_ctm_initialized()) ==> (old(assoc_unix_time()) <= assoc_unix_time());
    }
    spec module {
        apply GlobalWallClockIsMonotonic to *;
    }

    // **************** FUNCTION SPECIFICATIONS ****************

    spec fun initialize {
        aborts_if Signer::get_address(association) != CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS();
        aborts_if root_ctm_initialized();
        ensures root_ctm_initialized();
        ensures assoc_unix_time() == 0;
    }

    spec fun set_time_has_started {
        aborts_if Signer::get_address(association) != CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS();
        aborts_if !spec_is_genesis();
        aborts_if !root_ctm_initialized();
        aborts_if assoc_unix_time() != 0;
        ensures !spec_is_genesis();
    }

    spec fun update_global_time {
        aborts_if Signer::get_address(account) != CoreAddresses::SPEC_VM_RESERVED_ADDRESS();
        aborts_if !root_ctm_initialized();
        aborts_if (proposer == CoreAddresses::SPEC_VM_RESERVED_ADDRESS()) && (timestamp != assoc_unix_time());
        aborts_if (proposer != CoreAddresses::SPEC_VM_RESERVED_ADDRESS()) && !(timestamp > assoc_unix_time());
        ensures assoc_unix_time() == timestamp;
    }

    spec fun now_microseconds {
        aborts_if !exists<CurrentTimeMicroseconds>(CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS());
        ensures result == assoc_unix_time();
    }

    spec fun is_genesis {
        aborts_if false;
        ensures result == spec_is_genesis();
    }

    spec fun is_not_initialized {
        aborts_if false;
        ensures result == !root_ctm_initialized() || assoc_unix_time() == 0;
    }
}

}
