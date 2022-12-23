address 0x42 {
module M {
    struct R {}
    struct S {}
    struct Cup<T> { f1: T }

    public fun eat(r: R) {
        R{} = r
    }
}
}

script {
use 0x42::M::{S, R, Cup};

fun main<T: drop>(
    _s: &signer,
    _a0: T,
    _a1: vector<T>,
    _a2: vector<vector<T>>,
    _a3: S,
    _a4: R,
    _a5: Cup<u8>,
    _a6: Cup<T>,
    _a7: vector<S>,
) {
    abort 0
}
}
