//! new-transaction

module M {
    struct S has drop {
        a: u64,
        b: u64,
    }

    struct R has key, store {}
    struct Cup has key {
        a: u64,
        b: R,
    }

    public fun t0() {
        S { b: 1 / 0, a: fail(0) };
    }

    public fun t1() {
        S { b: 18446744073709551615 + 18446744073709551615, a: fail(0) };
    }

    public fun t2() {
        S { b: 0 - 1, a: fail(0) };
    }

    public fun t3() {
        S { b: 1 % 0, a: fail(0) };
    }

    public fun t4() {
        S { b: 18446744073709551615 * 18446744073709551615, a: fail(0) };
    }

    public fun t5(account: &signer) acquires R {
        move_to(account, Cup { b: move_from(0x0), a: fail(0) });
    }

    public fun t6(account: &signer) {
        move_to(account, Cup { b: R{}, a: 0 });
        S { b: mts(account), a: fail(0) };
    }

    fun fail(code: u64): u64 {
        abort code
    }

    fun mts(account: &signer): u64 {
        move_to(account, Cup { b: R{}, a: 0 });
        0
    }
}

//! new-transaction
// check: ARITHMETIC_ERROR
script {
use {{default}}::M;
fun main() {
  M::t0()
}
}

//! new-transaction
// check: ARITHMETIC_ERROR
script {
use {{default}}::M;
fun main() {
  M::t1()
}
}

//! new-transaction
// check: ARITHMETIC_ERROR
script {
use {{default}}::M;
fun main() {
  M::t2()
}
}

//! new-transaction
// check: ARITHMETIC_ERROR
script {
use {{default}}::M;
fun main() {
  M::t3()
}
}

//! new-transaction
// check: ARITHMETIC_ERROR
script {
use {{default}}::M;
fun main() {
  M::t4()
}
}

//! new-transaction
// check: MISSING_DATA
script {
use {{default}}::M;
fun main(account: signer) {
  M::t5(&account)
}
}

//! new-transaction
// check: RESOURCE_ALREADY_EXISTS
script {
use {{default}}::M;
fun main(account: signer) {
  M::t6(&account)
}
}
