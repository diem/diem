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

// check: ABORTED
// check: 22
// check: "::M::foo at offset 2"
