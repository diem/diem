module B {
    struct T has drop {g: u64}

    public fun new(g: u64): T {
        T { g }
    }

    public fun t(this: &T) {
        let g = &this.g;
        *g;
    }
}

//! new-transaction

script {
use {{default}}::B;

fun main() {
    let x = B::new(5);
    B::t(&x);
}
}
