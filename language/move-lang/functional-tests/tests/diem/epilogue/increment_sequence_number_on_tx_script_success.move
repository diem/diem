//! account: default, 1000000, 0

script {
fun main() {
}
}

// check: "Keep(EXECUTED)"


//! new-transaction
script {
use 0x1::DiemAccount;
use 0x1::Signer;

fun main(account: &signer) {
    let sender = Signer::address_of(account);
    assert(DiemAccount::sequence_number(sender) == 1, 42);
}
}
// check: "Keep(EXECUTED)"
