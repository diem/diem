module M {
    public fun foobar(cond: bool) {
        loop {
            loop {
                if (cond) break
            };
            if (cond) break
        }
    }
}

//! new-transaction
use {{default}}::M;

fun main() {
    M::foobar(true)
}
