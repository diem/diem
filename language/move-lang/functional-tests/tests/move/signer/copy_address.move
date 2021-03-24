//! account: alice
//! new-transaction
//! sender: alice

script {
use 0x1::Signer;
fun main(s: signer) {
    let s = &s;
    assert(*Signer::borrow_address(s) == Signer::address_of(s), 42);
    assert(Signer::address_of(s) == {{alice}}, 42)
}
}
