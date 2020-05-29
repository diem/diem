//! account: alice
//! account: bob

//! sender: alice
module M {
    use 0x0::Debug;

    public fun sum(n: u64): u64 {
        if (n < 2) {
            Debug::print_stack_trace();
            n
        } else {
            n + sum(n - 1)
        }
    }
}
// check: EXECUTED

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
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{bob}}::N;
use 0x0::Vector;
use 0x0::Debug;
use 0x0::LibraAccount;

fun main() {
    let v = Vector::empty();
    Vector::push_back(&mut v, true);
    Vector::push_back(&mut v, false);
    let r = Vector::borrow(&mut v, 1);
    let x = N::foo<bool, LibraAccount::T>();
    Debug::print(&x);
    _ = r;
}
}
// check: EXECUTED
