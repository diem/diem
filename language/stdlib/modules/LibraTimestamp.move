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
    use 0x1::CoreErrors;
    use 0x1::Signer;

    /// A singleton resource holding the current Unix time in microseconds
    resource struct CurrentTimeMicroseconds {
        microseconds: u64,
    }

    /// A singleton resource used to determine whether time has started. This
    /// is called at the end of genesis.
    resource struct TimeHasStarted {}

    spec module {
        /// Verify all functions in this module.
        pragma verify;

        /// All functions which do not have an `aborts_if` specification in this module are implicitly declared
        /// to never abort.
        pragma aborts_if_is_strict;
    }

    spec module {
        /// After genesis, the time stamp and time-has-started marker are published.
        invariant [global]
            is_operating() ==>
                exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()) &&
                exists<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        /// After genesis, time progresses monotonically.
        invariant update [global]
            is_operating() ==> old(spec_now_microseconds()) <= spec_now_microseconds();
    }


    /// Initializes the global wall clock time resource. This can only be called from genesis and with the
    /// root account.
    public fun initialize(lr_account: &signer) {
        assert(Self::is_genesis(), CoreErrors::NOT_GENESIS());
        // Operational constraint, only callable by the libra root account
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), CoreErrors::NOT_LIBRA_ROOT_ADDRESS());

        // TODO: Should the initialized value be passed in to genesis?
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        assert(!exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()), CoreErrors::ALREADY_INITIALIZED());
        move_to(lr_account, timer);
    }
    spec fun initialize {
        include AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        aborts_if spec_root_ctm_initialized() with CoreErrors::ALREADY_INITIALIZED();
        ensures spec_root_ctm_initialized();
        ensures spec_now_microseconds() == 0;
    }

    /// A helper to check whether the association root account has a CurrentTimeMicroseconds.
    spec define spec_root_ctm_initialized(): bool {
        exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis and with the root
    /// account.
    public fun set_time_has_started(lr_account: &signer) acquires CurrentTimeMicroseconds {
        assert(Self::is_genesis(), CoreErrors::NOT_GENESIS());
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), CoreErrors::NOT_LIBRA_ROOT_ADDRESS());

        // Current time must have been initialized.
        assert(
            exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()) && now_microseconds() == 0,
            CoreErrors::NOT_INITIALIZED()
        );
        assert(!exists<TimeHasStarted>(Signer::address_of(lr_account)), CoreErrors::NOT_GENESIS());
        move_to(lr_account, TimeHasStarted{});
    }
    spec fun set_time_has_started {
        /// This function can currently not be verified standalone. Once we publish the
        /// `TimeHasStarted` resource, all invariants in the system which describe the state of the
        /// system after genesis will be asserted. The caller need to establish a state where
        /// these assertions verify.
        pragma verify = false;
        include AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        aborts_if !spec_root_ctm_initialized() || spec_now_microseconds() != 0 with CoreErrors::NOT_INITIALIZED();
        ensures is_operating();
    }

    /// Helper functions for tests to reset the time-has-started, and pretend to be in genesis.
    /// > TODO(wrwg): we should have a capability which only tests can have to be able to call
    /// > this function.
    public fun reset_time_has_started_for_test() acquires TimeHasStarted {
        if (exists<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS())) {
            // Only do this if it actually exists.
            let TimeHasStarted{} = move_from<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        }
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        assert(is_operating(), CoreErrors::NOT_OPERATING());

        // Can only be invoked by LibraVM signer.
        assert(Signer::address_of(account) == CoreAddresses::VM_RESERVED_ADDRESS(), CoreErrors::NOT_VM_RESERVED_ADDRESS());

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let now = global_timer.microseconds;
        if (proposer == CoreAddresses::VM_RESERVED_ADDRESS()) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert(now == timestamp, CoreErrors::INVALID_INPUT());
        } else {
            // Normal block. Time must advance
            assert(now < timestamp, CoreErrors::INVALID_INPUT());
        };
        global_timer.microseconds = timestamp;
    }
    spec fun update_global_time {
        include AbortsIfNotOperating;
        include CoreAddresses::AbortsIfNotVM;
        let now = old(spec_now_microseconds());
        // TODO(wrwg): remove this assume by ensuring callers do not violate the condition
        aborts_if [assume]
            (if (proposer == CoreAddresses::VM_RESERVED_ADDRESS()) {
                now != timestamp
             } else  {
                now >= timestamp
             }
            )
            with CoreErrors::INVALID_INPUT();
        ensures spec_now_microseconds() == timestamp;
    }


    /// Gets the timestamp representing `now` in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        assert(exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()), CoreErrors::NOT_INITIALIZED());
        borrow_global<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()).microseconds
    }
    spec fun now_microseconds {
        pragma opaque;
        aborts_if !spec_root_ctm_initialized() with CoreErrors::NOT_INITIALIZED();
        ensures result == spec_now_microseconds();
    }
    spec define spec_now_microseconds(): u64 {
        global<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()).microseconds
    }

    /// Helper function to determine if Libra is in genesis state.
    public fun is_genesis(): bool {
        !exists<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Helper function to determine if Libra is operating. This is the same as `!is_genesis()` and is provided
    /// for convenience. Testing `is_operating()` is more frequent than `is_genesis()`.
    public fun is_operating(): bool {
        exists<TimeHasStarted>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Helper schema to specify that a function aborts if not in genesis.
    spec schema AbortsIfNotGenesis {
        aborts_if !is_genesis() with CoreErrors::NOT_GENESIS();
    }

    /// Helper schema to specify that a function aborts if not operating.
    spec schema AbortsIfNotOperating {
        aborts_if !is_operating() with CoreErrors::NOT_OPERATING();
    }

    /// Helper function to determine whether the CurrentTime has been not fully initialized.
    public fun is_not_initialized(): bool acquires CurrentTimeMicroseconds {
       !exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()) || now_microseconds() == 0
    }

    /// DEPRECATED: schema to check whether time can be accessed. This should be replaced by
    /// assertions that `is_operating()` is true.
    spec schema TimeAccessAbortsIf {
        aborts_if !spec_root_ctm_initialized();
    }

}

}
