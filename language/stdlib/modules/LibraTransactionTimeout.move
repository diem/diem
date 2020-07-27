address 0x1 {

module LibraTransactionTimeout {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraTimestamp;
    use 0x1::Roles;

    resource struct TTL {
        // Only transactions with timestamp in between block time and block time + duration would be accepted.
        duration_microseconds: u64,
    }

    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> is_initialized();
    }

    // U64_MAX / 1_000_000
    const MAX_TIMESTAMP: u64 = 18446744073709551615 / 1000000;
    const MICROS_MULTIPLIER: u64 = 1000000;
    const ONE_DAY_MICROS: u64 = 86400000000;

    const ETTL: u64 = 0;

    public fun initialize(lr_account: &signer) {
        LibraTimestamp::assert_genesis();
        // Operational constraint, only callable by the libra root account
        CoreAddresses::assert_libra_root(lr_account);
        assert(!is_initialized(), Errors::already_published(ETTL));
        // Currently set to 1day.
        move_to(lr_account, TTL {duration_microseconds: ONE_DAY_MICROS});
    }

    fun is_initialized(): bool {
        exists<TTL>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    // TODO (dd): is this called anywhere?
    public fun set_timeout(lr_account: &signer, new_duration: u64) acquires TTL {
        LibraTimestamp::assert_operating();
        Roles::assert_libra_root(lr_account);
        let timeout = borrow_global_mut<TTL>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        timeout.duration_microseconds = new_duration;
    }

    public fun is_valid_transaction_timestamp(timestamp: u64): bool acquires TTL {
        LibraTimestamp::assert_operating();
        // Reject timestamp greater than u64::MAX / 1_000_000.
        // This allows converting the timestamp from seconds to microseconds.
        if (timestamp > MAX_TIMESTAMP) {
          return false
        };

        let current_block_time = LibraTimestamp::now_microseconds();
        let timeout = borrow_global<TTL>(CoreAddresses::LIBRA_ROOT_ADDRESS()).duration_microseconds;
        let _max_txn_time = current_block_time + timeout;

        let txn_time_microseconds = timestamp * MICROS_MULTIPLIER;
        // TODO: Add LibraTimestamp::is_before_exclusive(&txn_time_microseconds, &max_txn_time)
        //       This is causing flaky test right now. The reason is that we will use this logic for AC, where its wall
        //       clock time might be out of sync with the real block time stored in StateStore.
        //       See details in issue #2346.
        current_block_time < txn_time_microseconds
    }
}
}
