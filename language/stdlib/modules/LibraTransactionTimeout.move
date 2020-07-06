address 0x1 {

module LibraTransactionTimeout {
  use 0x1::CoreAddresses;
  use 0x1::Signer;
  use 0x1::LibraTimestamp;
  use 0x1::Roles;

  resource struct TTL {
    // Only transactions with timestamp in between block time and block time + duration would be accepted.
    duration_microseconds: u64,
  }

    // U64_MAX / 1_000_000
    const MAX_TIMESTAMP: u64 = 18446744073709551615 / 1000000;
    const MICROS_MULTIPLIER: u64 = 1000000;
    const ONE_DAY_MICROS: u64 = 86400000000;

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ENOT_LIBRA_ROOT: u64 = 2;

  public fun initialize(association: &signer) {
    assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
    // Operational constraint, only callable by the Association address
    assert(Signer::address_of(association) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
    // Currently set to 1day.
    move_to(association, TTL {duration_microseconds: ONE_DAY_MICROS});
  }

  // TODO (dd): is this called anywhere?
  public fun set_timeout(
    lr_account: &signer,
    new_duration: u64,
    ) acquires TTL {
    assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
    let timeout = borrow_global_mut<TTL>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    timeout.duration_microseconds = new_duration;
  }

  public fun is_valid_transaction_timestamp(timestamp: u64): bool acquires TTL {
    // Reject timestamp greater than u64::MAX / 1_000_000
    if(timestamp > MAX_TIMESTAMP) {
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
