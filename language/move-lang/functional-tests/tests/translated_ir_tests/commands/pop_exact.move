module A {
    fun three(): (u64, u64, u64) {
        (0, 1, 2)
    }

    public fun pop() {
        (_, _, _) = three();
    }
}

//! new-transaction

script {
use {{default}}::A;

fun main() {
    A::pop();
}
}
