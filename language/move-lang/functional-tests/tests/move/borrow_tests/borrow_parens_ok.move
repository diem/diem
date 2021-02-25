module M {
    struct T has drop {v: u64}

    public fun new(v: u64) : T {
        T { v }
    }

    public fun value(this: &T) : u64 {
        //borrow of move
        let f = (move this).v;
        f
    }
}

//! new-transaction

script {
use {{default}}::M;

fun main() {
    let x = M::new(5);
    let x_ref = &x;
    let y = M::value(x_ref);
    assert(y == 5, 42);
}
}
