address 0x1 {

module LibraTransactionTimeout {
  use 0x1::CoreAddresses;
  use 0x1::Signer;
  use 0x1::LibraTimestamp;
  use 0x1::Roles::{Capability, AssociationRootRole};

  resource struct TTL {
    // Only transactions with timestamp in between block time and block time + duration would be accepted.
    duration_microseconds: u64,
  }

  public fun initialize(association: &signer) {
    // Operational constraint, only callable by the Association address
    assert(Signer::address_of(association) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 1);
    // Currently set to 1day.
    move_to(association, TTL {duration_microseconds: 86400000000});
  }

  public fun set_timeout(_: &Capability<AssociationRootRole>, new_duration: u64) acquires TTL {
    let timeout = borrow_global_mut<TTL>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
    timeout.duration_microseconds = new_duration;
  }

  public fun is_valid_transaction_timestamp(timestamp: u64): bool acquires TTL {
    // Reject timestamp greater than u64::MAX / 1_000_000;
    if(timestamp > 9223372036854) {
      return false
    };

    let current_block_time = LibraTimestamp::now_microseconds();
    let timeout = borrow_global<TTL>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()).duration_microseconds;
    let _max_txn_time = current_block_time + timeout;

    let txn_time_microseconds = timestamp * 1000000;
    // TODO: Add LibraTimestamp::is_before_exclusive(&txn_time_microseconds, &max_txn_time)
    //       This is causing flaky test right now. The reason is that we will use this logic for AC, where its wall
    //       clock time might be out of sync with the real block time stored in StateStore.
    //       See details in issue #2346.
    current_block_time < txn_time_microseconds
  }
}

}
