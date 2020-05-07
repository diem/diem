address 0x1 {
module X {
    struct U {}
    resource struct R {}
}

module M {
    use 0x1::X::{U, U as U2};

    fun f(u: U, r: R) {
        g(u, r)
    }

    fun g(u: U2, r: R2) {
        f(u, r)
    }

    use 0x1::X::{R, R as R2};
}

}
