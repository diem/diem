address 0x2 {
module X {
    struct U {}
    struct R {}
}

module M {
    use 0x2::X::{U, U as U2};

    fun U() {}
    fun U2() {}
    fun R() {}
    fun R2() {}

    use 0x2::X::{R, R as R2};
}

}
