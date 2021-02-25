address 0x2 {
module X {
    struct U {}
    struct R {}
}

module M {
    use 0x2::X::{U, U as U2};

    struct U {}
    struct U2 {}
    struct R2 {}
    struct R {}

    use 0x2::X::{R, R as R2};
}

}
