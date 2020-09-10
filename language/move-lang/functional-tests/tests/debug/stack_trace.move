//! account: alice
//! account: bob

//! sender: alice
module M {
    use 0x1::Debug;

    public fun sum(n: u64): u64 {
        if (n < 2) {
            Debug::print_stack_trace();
            n
        } else {
            n + sum(n - 1)
        }
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
module N {
    use {{alice}}::M;

    public fun foo<T1, T2>(): u64 {
        let x = 3;
        let y = &mut x;
        let z = M::sum(4);
        _ = y;
        z
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{bob}}::N;
use 0x1::Vector;
use 0x1::Debug;
use 0x1::LibraAccount::LibraAccount;

fun main() {
    let v = Vector::empty();
    Vector::push_back(&mut v, true);
    Vector::push_back(&mut v, false);
    let r = Vector::borrow(&mut v, 1);
    let x = N::foo<bool, LibraAccount>();
    Debug::print(&x);
    _ = r;
}
}
// check: "Keep(EXECUTED)"
