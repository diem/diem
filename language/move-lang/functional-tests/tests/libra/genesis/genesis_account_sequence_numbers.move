script {
use 0x1::LibraAccount;
use 0x1::CoreAddresses;

fun main() {
  // check that the sequence number of the Association account (which sent the genesis txn) has been
  // incremented...
  assert(LibraAccount::sequence_number(CoreAddresses::LIBRA_ROOT_ADDRESS()) == 1, 66);
}
}
