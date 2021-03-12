//! new-transaction

module {{default}}::X {
    struct T has drop {}
    public fun new(): T {
        T {}
    }
}

//! new-transaction

module {{default}}::Y {
    use {{default}}::X;
    public fun foo(): X::T {
        X::new()
    }
}


//! new-transaction
script {
use {{default}}::Y;

fun main() {
    Y::foo();
}
}
