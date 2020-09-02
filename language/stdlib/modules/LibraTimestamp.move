address 0x1 {

/// This module keeps a global wall clock that stores the current Unix time in microseconds.
/// It interacts with the other modules in the following ways:
///
/// * Genesis: to initialize the timestamp
/// * VASP: to keep track of when credentials expire
/// * LibraSystem, LibraAccount, LibraConfig: to check if the current state is in the genesis state
/// * LibraBlock: to reach consensus on the global wall clock time
/// * AccountLimits: to limit the time of account limits
///
module LibraTimestamp {
    use 0x1::CoreAddresses;
    use 0x1::Errors;

    /// A singleton resource holding the current Unix time in microseconds
    resource struct CurrentTimeMicroseconds {
        microseconds: u64,
    }

    /// Conversion factor between seconds and microseconds
    const MICRO_CONVERSION_FACTOR: u64 = 1000000;

    /// The blockchain is not in the genesis state anymore
    const ENOT_GENESIS: u64 = 0;
    /// The blockchain is not in an operating state yet
    const ENOT_OPERATING: u64 = 1;
    /// An invalid timestamp was provided
    const ETIMESTAMP: u64 = 2;

    spec module {
        /// Verify all functions in this module.
        pragma verify;

        /// All functions which do not have an `aborts_if` specification in this module are implicitly declared
        /// to never abort.
        pragma aborts_if_is_strict;
    }

    spec module {
        /// After genesis, time progresses monotonically.
        invariant update [global]
            old(is_operating()) ==> old(spec_now_microseconds()) <= spec_now_microseconds();
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis and with the root
    /// account.
    public fun set_time_has_started(lr_account: &signer) {
        assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        move_to(lr_account, timer);
    }
    spec fun set_time_has_started {
        include AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        ensures is_operating();
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        assert_operating();
        // Can only be invoked by LibraVM signer.
        CoreAddresses::assert_vm(account);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let now = global_timer.microseconds;
        if (proposer == CoreAddresses::VM_RESERVED_ADDRESS()) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert(now == timestamp, Errors::invalid_argument(ETIMESTAMP));
        } else {
            // Normal block. Time must advance
            assert(now < timestamp, Errors::invalid_argument(ETIMESTAMP));
        };
        global_timer.microseconds = timestamp;
    }
    spec fun update_global_time {
        include AbortsIfNotOperating;
        include CoreAddresses::AbortsIfNotVM;
        let now = spec_now_microseconds();
        // TODO(wrwg): remove this assume by ensuring callers do not violate the condition
        aborts_if [assume]
            (if (proposer == CoreAddresses::VM_RESERVED_ADDRESS()) {
                now != timestamp
             } else  {
                now >= timestamp
             }
            )
            with Errors::INVALID_ARGUMENT;
        ensures spec_now_microseconds() == timestamp;
    }

    /// Gets the current time in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        assert_operating();
        borrow_global<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()).microseconds
    }
    spec fun now_microseconds {
        pragma opaque;
        include AbortsIfNotOperating;
        ensures result == spec_now_microseconds();
    }
    spec define spec_now_microseconds(): u64 {
        global<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()).microseconds
    }

    /// Gets the current time in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeMicroseconds {
        now_microseconds() / MICRO_CONVERSION_FACTOR
    }
    spec fun now_seconds {
        pragma opaque;
        include AbortsIfNotOperating;
        ensures result == spec_now_microseconds() /  MICRO_CONVERSION_FACTOR;
    }
    spec define spec_now_seconds(): u64 {
        global<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS()).microseconds / MICRO_CONVERSION_FACTOR
    }

    /// Helper function to determine if Libra is in genesis state.
    public fun is_genesis(): bool {
        !exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Helper function to assert genesis state.
    public fun assert_genesis() {
        assert(is_genesis(), Errors::invalid_state(ENOT_GENESIS));
    }
    spec fun assert_genesis {
        pragma opaque = true;
        include AbortsIfNotGenesis;
    }

    /// Helper schema to specify that a function aborts if not in genesis.
    spec schema AbortsIfNotGenesis {
        aborts_if !is_genesis() with Errors::INVALID_STATE;
    }

    /// Helper function to determine if Libra is operating. This is the same as `!is_genesis()` and is provided
    /// for convenience. Testing `is_operating()` is more frequent than `is_genesis()`.
    public fun is_operating(): bool {
        exists<CurrentTimeMicroseconds>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Helper function to assert operating (!genesis) state.
    public fun assert_operating() {
        assert(is_operating(), Errors::invalid_state(ENOT_OPERATING));
    }
    spec fun assert_operating {
        pragma opaque = true;
        include AbortsIfNotOperating;
    }

    /// Helper schema to specify that a function aborts if not operating.
    spec schema AbortsIfNotOperating {
        aborts_if !is_operating() with Errors::INVALID_STATE;
    }
}

}
