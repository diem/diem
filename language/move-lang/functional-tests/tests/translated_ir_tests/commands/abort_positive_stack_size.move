module M {

    public fun foo(): u64 {
        0
    }

}

//! new-transaction

script {
use {{default}}::M;

fun main() {
    M::foo();
    abort 0
}
}

// not: VerificationFailure

// check: "Keep(ABORTED { code: 0,"
