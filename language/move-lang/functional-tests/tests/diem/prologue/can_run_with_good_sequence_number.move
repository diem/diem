//! account: alice, 10000, 10

//! sequence-number: 10
//! sender: alice
script {
use DiemFramework::DiemAccount;

fun main() {
    assert(DiemAccount::sequence_number(@{{alice}}) == 10, 72);
}
}
// check: "Keep(EXECUTED)"
