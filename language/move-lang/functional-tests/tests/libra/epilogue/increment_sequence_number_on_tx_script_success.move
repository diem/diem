//! account: default, 1000000, 0

script {
fun main() {
}
}

// check: "Keep(EXECUTED)"


//! new-transaction
script {
use 0x1::LibraAccount;
use 0x1::Signer;

fun main(account: &signer) {
    let sender = Signer::address_of(account);
    assert(LibraAccount::sequence_number(sender) == 1, 42);
}
}
// check: "Keep(EXECUTED)"
