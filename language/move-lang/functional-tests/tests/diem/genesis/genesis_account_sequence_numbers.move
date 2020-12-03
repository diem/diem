script {
use 0x1::DiemAccount;
use 0x1::CoreAddresses;

fun main() {
  // check that the sequence number of the Association account (which sent the genesis txn) has been
  // incremented...
  assert(DiemAccount::sequence_number(CoreAddresses::DIEM_ROOT_ADDRESS()) == 1, 66);
}
}
