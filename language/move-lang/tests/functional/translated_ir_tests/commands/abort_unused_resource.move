module M {
    resource struct T {}
    public fun new(): T {
        T{}
    }
}

//! new-transaction
use {{default}}::M;

fun main() {
    let t = M::new();
    &t;
    // z is allowed to be unused
    abort 0
}

// not: VerificationFailure

// check: ABORTED
// check: 0
