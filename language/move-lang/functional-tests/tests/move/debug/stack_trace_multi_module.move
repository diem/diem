//! account: alice
//! account: bob

//! sender: alice
module M {
    use 0x1::Debug;

    public fun bar() {
        Debug::print_stack_trace();
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
module N {
    use {{alice}}::M;
    use 0x1::Libra;
    use 0x1::Coin1::Coin1;

    public fun foo() {
        let x = Libra::zero<Coin1>();
        M::bar();
        Libra::destroy_zero(x);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{bob}}::N;

fun main() {
    N::foo();
}
}
// check: "Keep(EXECUTED)"
