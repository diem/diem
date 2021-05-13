//! account: alice
//! new-transaction
//! sender: alice

script {
use Std::Signer;
fun main(s: signer) {
    let s = &s;
    assert(Signer::borrow_address(s) == &@{{alice}}, 42)
}
}
