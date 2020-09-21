module M {
    public fun foo() {
        abort 22
    }
}

//! new-transaction
script {
use {{default}}::M;

fun main() {
    M::foo()
}
}

// check: "Keep(ABORTED { code: 22,"
