//! account: alice
//! new-transaction
//! sender: alice

script {
use 0x1::Signer;
fun main(s: signer) {
    let s = &s;
    assert(Signer::borrow_address(s) == &{{alice}}, 42)
}
}
