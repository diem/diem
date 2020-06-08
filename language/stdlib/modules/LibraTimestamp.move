address 0x0 {

module LibraTimestamp {
    use 0x0::Signer;
    use 0x0::Transaction;

    // A singleton resource holding the current Unix time in microseconds
    resource struct CurrentTimeMicroseconds {
        microseconds: u64,
    }

    // Initialize the global wall clock time resource.
    public fun initialize(association: &signer) {
        // Only callable by the Association address
        Transaction::assert(Signer::address_of(association) == 0xA550C18, 1);

        // TODO: Should the initialized value be passed in to genesis?
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        move_to(association, timer);
    }

    // Update the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        // Can only be invoked by LibraVM privilege.
        Transaction::assert(Signer::address_of(account) == 0x0, 33);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(0xA550C18);
        if (proposer == 0x0) {
            // NIL block with null address as proposer. Timestamp must be equal.
            Transaction::assert(timestamp == global_timer.microseconds, 5001);
        } else {
            // Normal block. Time must advance
            Transaction::assert(global_timer.microseconds < timestamp, 5001);
        };
        global_timer.microseconds = timestamp;
    }

    // Get the timestamp representing `now` in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        borrow_global<CurrentTimeMicroseconds>(0xA550C18).microseconds
    }

    // Helper function to determine if the blockchain is at genesis state.
    public fun is_genesis(): bool acquires CurrentTimeMicroseconds {
        !exists<CurrentTimeMicroseconds>(0xA550C18) || now_microseconds() == 0
    }

    /**
    **************** SPECIFICATIONS ****************

    This module keeps a global wall clock that stores the current Unix time in microseconds.
    It interacts with the other modules in the following ways:
        * Genesis: to initialize the timestamp
        * VASP: to keep track of when credentials expire
        * LibraSystem, LibraAccount, LibraConfig: to check if the current state is the genesis state
        * LibraBlock: to reach consensus on the global wall clock time
        * AccountLimits: to limit the time of account limits
        * LibraTransactionTimeout: to determine whether a transaction is still valid
    */

    /// # Module specification

    spec module {
        /// Verify all functions in this module.
        pragma verify = true;

        /// The association root address.
        define root_address(): address {
            0xA550C18
        }
        /// The nil account with VM privilege.
        define null_address(): address {
            0x0
        }
        /// True if the association root account has a CurrentTimeMicroseconds.
        define root_ctm_initialized(): bool {
            exists<CurrentTimeMicroseconds>(root_address())
        }
        /// Auxiliary function to get the association's Unix time in microseconds.
        define assoc_unix_time(): u64 {
            global<CurrentTimeMicroseconds>(root_address()).microseconds
        }
    }

    /// ## Management of the global wall clock time

    spec module {
        /// **Informally:** Only the root address has a CurrentTimeMicroseconds struct.
        define only_root_addr_has_ctm(): bool {
            all(domain<address>(), |addr|
                exists<CurrentTimeMicroseconds>(addr)
                    ==> addr == root_address())
        }
    }
    spec schema OnlyRootAddressHasTimestamp {
        /// Base case of the induction step before the root
        /// account is initialized with the CurrentTimeMicroseconds.
        ///
        /// **Informally:** If the root account hasn't been initialized with
        /// CurrentTimeMicroseconds, then it should not exist.
        invariant module !root_ctm_initialized()
                            ==> all(domain<address>(), |addr| !exists<CurrentTimeMicroseconds>(addr));
        /// Induction hypothesis for invariant after initialization.
        ///
        /// **Informally:** Only the association account has a timestamp `CurrentTimeMicroseconds`.
        invariant module root_ctm_initialized() ==> only_root_addr_has_ctm();
        /// **Informally:** If the CurrentTimeMicroseconds is initialized, it stays initialized.
        ensures old(root_ctm_initialized()) ==> root_ctm_initialized();
    }
    spec module {
        /// Apply `OnlyRootAddressHasTimestamp` to all functions.
        /// No other functions should contain the CurrentTimeMicroseconds struct.
        apply OnlyRootAddressHasTimestamp to *;
    }
    spec fun initialize {
        /// The association / root account creates a timestamp struct
        /// under its account during initialization.
        aborts_if Signer::get_address(association) != root_address();
        aborts_if root_ctm_initialized();
        ensures root_ctm_initialized();
        ensures assoc_unix_time() == 0;
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Global wall clock time progression

    spec schema GlobalWallClockIsMonotonic {
        /// **Informally:** The global wall clock time never decreases.
        ensures old(root_ctm_initialized()) ==> (old(assoc_unix_time()) <= assoc_unix_time());
    }
    spec module {
        /// Apply `GlobalWallClockIsMonotonic` to all functions.
        /// The global wall clock time should only increase.
        apply GlobalWallClockIsMonotonic to *;
    }
    spec fun update_global_time {
        /// Used to update every proposers' global wall
        /// clock time by consensus.
        aborts_if Signer::get_address(account) != null_address();
        aborts_if !root_ctm_initialized();
        aborts_if (proposer == null_address()) && (timestamp != assoc_unix_time());
        aborts_if (proposer != null_address()) && !(timestamp > assoc_unix_time());
        ensures assoc_unix_time() == timestamp;
    }

    // Switch documentation context back to module level.
    spec module {}

    spec fun now_microseconds {
        /// Returns the global wall clock time if it exists.
        aborts_if !exists<CurrentTimeMicroseconds>(root_address());
        ensures result == assoc_unix_time();
    }

    spec fun is_genesis {
        /// Returns whether or not it is the beginning of time.
        aborts_if false;
        ensures (result == !root_ctm_initialized() || assoc_unix_time() == 0);
    }
}

}
