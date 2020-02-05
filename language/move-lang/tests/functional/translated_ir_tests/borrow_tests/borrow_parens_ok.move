module M {
    struct T{v: u64}

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

use {{default}}::M;

fun main() {
    let x = M::new(5);
    let x_ref = &x;
    let y = M::value(x_ref);
    0x0::Transaction::assert(y == 5, 42);
}
