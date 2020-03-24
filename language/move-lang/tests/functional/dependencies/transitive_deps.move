//! new-transaction

module X {
    struct T {}
    public fun new(): T {
        T {}
    }
}

//! new-transaction

module Y {
    use {{default}}::X;
    public fun foo(): X::T {
        X::new()
    }
}


//! new-transaction

use {{default}}::Y;

fun main() {
    Y::foo();
}
