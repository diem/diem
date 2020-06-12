//! account: alice
//! new-transaction
//! sender: alice

script {
use 0x0::Signer;
fun main(s: &signer) {
    assert(*Signer::borrow_address(s) == Signer::address_of(s), 42);
    assert(Signer::address_of(s) == {{alice}}, 42)
}
}
