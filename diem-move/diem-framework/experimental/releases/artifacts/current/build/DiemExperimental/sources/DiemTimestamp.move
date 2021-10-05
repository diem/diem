/// This module keeps a global wall clock that stores the current Unix time in microseconds.
/// It interacts with the other modules in the following ways:
///
/// * Genesis: to initialize the timestamp
/// * VASP: to keep track of when credentials expire
/// * DiemSystem, DiemAccount, DiemConfig: to check if the current state is in the genesis state
/// * DiemBlock: to reach consensus on the global wall clock time
/// * AccountLimits: to limit the time of account limits
///
/// This module moreover enables code to assert that it is running in genesis (`Self::assert_genesis`) or after
/// genesis (`Self::assert_operating`). These are essentially distinct states of the system. Specifically,
/// if `Self::assert_operating` succeeds, assumptions about invariants over the global state can be made
/// which reflect that the system has been successfully initialized.
module DiemFramework::DiemTimestamp {
    use DiemFramework::CoreAddresses;
    use Std::Errors;
    friend DiemFramework::Genesis;

    /// A singleton resource holding the current Unix time in microseconds
    struct CurrentTimeMicroseconds has key {
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

    /// Marks that time has started and genesis has finished. This can only be called from genesis and with the root
    /// account.
    public(friend) fun set_time_has_started(dr_account: &signer) {
        assert_genesis();
        CoreAddresses::assert_diem_root(dr_account);
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        move_to(dr_account, timer);
    }
    spec set_time_has_started {
        /// This function can't be verified on its own and has to be verified in the context of Genesis execution.
        ///
        /// After time has started, all invariants guarded by `DiemTimestamp::is_operating` will become activated
        /// and need to hold.
        pragma delegate_invariants_to_caller;
        include AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotDiemRoot{account: dr_account};
        ensures is_operating();
    }

    // TODO: this is for both df-cli and the unit-test for df
    // - df-cli, as a few test cases in df-cli uses a customized genesis module and that module needs  to invoke
    //   `set_time_has_started` in order to complete the genesis process. Until we find a way to solve this issue, this
    //   temporary function will stay here.
    // - this is also needed for diem-framework unit test `DiemTimestampTests`. And once the above issue for df-cli is
    //   resolved, we can mark this function #[test_only]
    public fun set_time_has_started_for_testing(dr_account: &signer) {
        set_time_has_started(dr_account);
    }
    spec set_time_has_started_for_testing {
        pragma verify = false;
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        assert_operating();
        // Can only be invoked by DiemVM signer.
        CoreAddresses::assert_vm(account);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(@DiemRoot);
        let now = global_timer.microseconds;
        if (proposer == @VMReserved) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert!(now == timestamp, Errors::invalid_argument(ETIMESTAMP));
        } else {
            // Normal block. Time must advance
            assert!(now < timestamp, Errors::invalid_argument(ETIMESTAMP));
        };
        global_timer.microseconds = timestamp;
    }
    spec update_global_time {
        pragma opaque;
        modifies global<CurrentTimeMicroseconds>(@DiemRoot);

        let now = spec_now_microseconds();
        let post post_now = spec_now_microseconds();

        /// Conditions unique for abstract and concrete version of this function.
        include AbortsIfNotOperating;
        include CoreAddresses::AbortsIfNotVM;
        ensures post_now == timestamp;

        /// Conditions we only check for the implementation, but do not pass to the caller.
        aborts_if [concrete]
            (if (proposer == @VMReserved) {
                now != timestamp
             } else  {
                now >= timestamp
             }
            )
            with Errors::INVALID_ARGUMENT;
    }

    /// Gets the current time in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        assert_operating();
        borrow_global<CurrentTimeMicroseconds>(@DiemRoot).microseconds
    }
    spec now_microseconds {
        pragma opaque;
        include AbortsIfNotOperating;
        ensures result == spec_now_microseconds();
    }
    spec fun spec_now_microseconds(): u64 {
        global<CurrentTimeMicroseconds>(@DiemRoot).microseconds
    }

    /// Gets the current time in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeMicroseconds {
        now_microseconds() / MICRO_CONVERSION_FACTOR
    }
    spec now_seconds {
        pragma opaque;
        include AbortsIfNotOperating;
        ensures result == spec_now_microseconds() /  MICRO_CONVERSION_FACTOR;
    }
    spec fun spec_now_seconds(): u64 {
        global<CurrentTimeMicroseconds>(@DiemRoot).microseconds / MICRO_CONVERSION_FACTOR
    }

    /// Helper function to determine if Diem is in genesis state.
    public fun is_genesis(): bool {
        !exists<CurrentTimeMicroseconds>(@DiemRoot)
    }

    /// Helper function to assert genesis state.
    public fun assert_genesis() {
        assert!(is_genesis(), Errors::invalid_state(ENOT_GENESIS));
    }
    spec assert_genesis {
        pragma opaque = true;
        include AbortsIfNotGenesis;
    }

    /// Helper schema to specify that a function aborts if not in genesis.
    spec schema AbortsIfNotGenesis {
        aborts_if !is_genesis() with Errors::INVALID_STATE;
    }

    /// Helper function to determine if Diem is operating. This is the same as `!is_genesis()` and is provided
    /// for convenience. Testing `is_operating()` is more frequent than `is_genesis()`.
    public fun is_operating(): bool {
        exists<CurrentTimeMicroseconds>(@DiemRoot)
    }

    /// Helper function to assert operating (!genesis) state.
    public fun assert_operating() {
        assert!(is_operating(), Errors::invalid_state(ENOT_OPERATING));
    }
    spec assert_operating {
        pragma opaque = true;
        include AbortsIfNotOperating;
    }

    /// Helper schema to specify that a function aborts if not operating.
    spec schema AbortsIfNotOperating {
        aborts_if !is_operating() with Errors::INVALID_STATE;
    }

    // ====================
    // Module Specification
    spec module {} // switch documentation context to module level

    spec module {
        /// After genesis, `CurrentTimeMicroseconds` is published forever
        invariant is_operating() ==> exists<CurrentTimeMicroseconds>(@DiemRoot);

        /// After genesis, time progresses monotonically.
        invariant update
            old(is_operating()) ==> old(spec_now_microseconds()) <= spec_now_microseconds();
    }

    spec module {
        /// All functions which do not have an `aborts_if` specification in this module are implicitly declared
        /// to never abort.
        pragma aborts_if_is_strict;
    }

}
