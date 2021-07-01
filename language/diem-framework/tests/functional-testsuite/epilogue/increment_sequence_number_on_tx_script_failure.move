//! account: default, 1000000, 0

script {
// When a transaction aborts, the sequence number should still be bumped up.
fun main() {
    assert(false, 77);
}
}
// check: "Keep(ABORTED { code: 77,"


//! new-transaction
script {
use 0x1::DiemAccount;
use 0x1::Signer;

fun main(account: signer) {
    let account = &account;
    let sender = Signer::address_of(account);
    assert(DiemAccount::sequence_number(sender) == 1, 42);
}
}
// check: "Keep(EXECUTED)"
