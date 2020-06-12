module A {
    struct T{v: u64}

    public fun new(v: u64): T {
        T { v }
    }

    public fun value(this: T): u64 {
        this.v
    }
}

//! new-transaction

script {
use {{default}}::A;
fun main() {
    let x = A::new(5);
    let x_ref = A::value(x);
    assert(x_ref == 5, 42);
}
}
