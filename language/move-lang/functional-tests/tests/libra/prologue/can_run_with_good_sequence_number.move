//! account: alice, 10000, 10

//! sequence-number: 10
//! sender: alice
script {
use 0x1::LibraAccount;

fun main() {
    assert(LibraAccount::sequence_number({{alice}}) == 10, 72);
}
}
// check: "Keep(EXECUTED)"
