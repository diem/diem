module M {
    resource struct R {}
    struct S {}
    struct Cup<T> {}

    public fun eat(r: R) {
        R{} = r
    }
}

//! new-transaction

script {
use {{default}}::M;

fun main(
    no1: M::R,
    no2: M::S,
    no3: M::Cup<u64>,
    no4: M::Cup<M::S>,
) {

}
}

// check: error
// check: M::R

// check: error
// check: M::S

// check: error
// check: M::Cup<u64>

// check: error
// check: M::Cup<
// check: M::S>
