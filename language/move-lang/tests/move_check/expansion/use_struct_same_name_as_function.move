address 0x1 {
module X {
    struct U {}
    resource struct R {}
}

module M {
    use 0x1::X::{U, U as U2};

    fun U() {}
    fun U2() {}
    fun R() {}
    fun R2() {}

    use 0x1::X::{R, R as R2};
}

}
