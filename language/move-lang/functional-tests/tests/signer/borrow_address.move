//! account: alice
//! new-transaction
//! sender: alice

script {
use 0x0::Signer;
fun main(s: &signer) {
    0x0::Transaction::assert(Signer::borrow_address(s) == &{{alice}}, 42)
}
}
