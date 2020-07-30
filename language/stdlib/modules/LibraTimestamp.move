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

    const EINVALID_SINGLETON_ADDRESS: u64 = 0;
    const ETIME_NOT_INITIALIZED: u64 = 1;
    const ENOT_VM: u64 = 2;
    const EINVALID_TIMESTAMP: u64 = 3;
    const EGENESIS_ONLY: u64 = 4;

    /// Initializes the global wall clock time resource. This can only be called from genesis.
    public fun initialize(lr_account: &signer) {
        // Operational constraint, only callable by the libra root account
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);

        // TODO: Should the initialized value be passed in to genesis?
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        move_to(lr_account, timer);
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis.
    public fun set_time_has_started(lr_account: &signer) acquires CurrentTimeMicroseconds {
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);

        // Current time must have been initialized.
        assert(
            exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()) && now_microseconds() == 0,
            ETIME_NOT_INITIALIZED
        );
        assert(!exists<TimeHasStarted>(Signer::address_of(lr_account)), EGENESIS_ONLY);
        move_to(lr_account, TimeHasStarted{});
    }

    /// Helper functions for tests to reset the time-has-started, and pretend to be in genesis.
    /// > TODO(wrwg): we should have a capability which only tests can have to be able to call
    /// > this function.
    public fun reset_time_has_started_for_test() acquires TimeHasStarted {
        let TimeHasStarted{} = move_from<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        // Can only be invoked by LibraVM privilege.
        assert(Signer::address_of(account) == CoreAddresses::VM_RESERVED_ADDRESS(), ENOT_VM);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        if (proposer == CoreAddresses::VM_RESERVED_ADDRESS()) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert(timestamp == global_timer.microseconds, EINVALID_TIMESTAMP);
        } else {
            // Normal block. Time must advance
            assert(global_timer.microseconds < timestamp, EINVALID_TIMESTAMP);
        };
        global_timer.microseconds = timestamp;
    }

    /// Gets the timestamp representing `now` in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        borrow_global<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()).microseconds
    }

    /// Helper function to determine if the blockchain is in genesis state.
    public fun is_genesis(): bool {
        !exists<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Helper function to determine whether the CurrentTime has been initialized.
    public fun is_not_initialized(): bool acquires CurrentTimeMicroseconds {
       !exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()) || now_microseconds() == 0
    }

    // **************** GLOBAL SPECIFICATION ****************

    /// # Module specification

    spec module {
        /// Verify all functions in this module.
        pragma verify = true;

        /// Specification version of the `Self::is_genesis` function.
        define spec_is_genesis(): bool {
            !exists<TimeHasStarted>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }

        /// Helper to express !is_genesis.
        define spec_is_up(): bool {
            !spec_is_genesis()
        }

        /// Specification version of the `Self::is_not_initialized` function.
        define spec_is_not_initialized(): bool {
            !spec_root_ctm_initialized() || spec_now_microseconds() == 0
        }

        /// True if the association root account has a CurrentTimeMicroseconds.
        define spec_root_ctm_initialized(): bool {
            exists<CurrentTimeMicroseconds>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }

        /// Auxiliary function to get the association's Unix time in microseconds.
        define spec_now_microseconds(): u64 {
            global<CurrentTimeMicroseconds>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()).microseconds
        }
    }

    /// ## Persistence of Initialization

    spec module {
        /// After genesis, the time stamp and time-has-started marker stay published.
        invariant [global]
            spec_is_up() ==>
                exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()) &&
                exists<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }

    /// ## Global Clock Time Progression

    spec module {
        /// After genesis, time progresses monotonically.
        invariant update [global]
            spec_is_up() ==> old(spec_now_microseconds()) <= spec_now_microseconds();
    }

    // **************** FUNCTION SPECIFICATIONS ****************

    spec fun initialize {
        aborts_if Signer::spec_address_of(lr_account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if spec_root_ctm_initialized();
        ensures spec_root_ctm_initialized();
        ensures spec_now_microseconds() == 0;
    }

    spec fun set_time_has_started {
        /// This function is not directly verified but only in its calling context. Once it publishes
        /// the `TimeHasStarted` resource, all invariants in the system which describe the resource state
        /// after genesis will be asserted. This function is only called from genesis, which must
        /// establish a state which passes the verification steps implied by calling this function.
        pragma verify = false;
        aborts_if Signer::spec_address_of(lr_account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if !spec_is_genesis();
        aborts_if !spec_root_ctm_initialized();
        aborts_if spec_now_microseconds() != 0;
        ensures !spec_is_genesis();
    }

    spec fun update_global_time {
        pragma assume_no_abort_from_here = true;
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_VM_RESERVED_ADDRESS();
        aborts_if !spec_root_ctm_initialized();
        aborts_if (proposer == CoreAddresses::SPEC_VM_RESERVED_ADDRESS()) && (timestamp != spec_now_microseconds());
        aborts_if (proposer != CoreAddresses::SPEC_VM_RESERVED_ADDRESS()) && !(timestamp > spec_now_microseconds());
        ensures spec_now_microseconds() == timestamp;
    }

    spec fun now_microseconds {
        include TimeAccessAbortsIf;
    }

    spec fun is_genesis {
        aborts_if false;
        ensures result == spec_is_genesis();
    }

    spec fun is_not_initialized {
        aborts_if false;
        ensures result == spec_is_not_initialized();
    }

    spec schema TimeAccessAbortsIf {
        aborts_if !exists<CurrentTimeMicroseconds>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS());
    }
}

}
