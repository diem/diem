module M {

    public fun foo(): u64 {
        0
    }

}

//! new-transaction

use {{default}}::M;

fun main() {
    M::foo();
    abort 0
}

// not: VerificationFailure

// check: ABORTED
// check: 0
