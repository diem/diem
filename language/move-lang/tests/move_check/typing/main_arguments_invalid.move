address 0x42 {
module M {
    resource struct R {}
    struct S {}
    struct Cup<T> {}

    public fun eat(r: R) {
        R{} = r
    }
}
}

script {
use 0x42::M::{S, R, Cup};

fun main<T: copyable>(
    a0: T,
    a1: vector<T>,
    a2: vector<vector<T>>,
    a3: S,
    a4: R,
    a5: Cup<u8>,
    a6: Cup<T>,
    a7: vector<S>,
) {

}
}
