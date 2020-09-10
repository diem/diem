module A {

    struct T{v: u64}

    struct K{f: T}

    public fun new_T(v: u64) : T {
        T { v }
    }

    public fun new_K(f: T) : K {
        K { f }
    }

    public fun value(this: &K) : u64 {
        this.f.v
    }
}

//! new-transaction

script {
use {{default}}::A;

fun main() {
    let x = A::new_T(2);
    let y = A::new_K(x);
    let z = A::value(&y);
    assert(z == 2, 42);
}
}
