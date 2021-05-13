script {
use DiemFramework::DiemAccount;

fun main() {
  // check that the sequence number of the Association account (which sent the genesis txn) has been
  // incremented...
  assert(DiemAccount::sequence_number(@DiemRoot) == 1, 66);
}
}
