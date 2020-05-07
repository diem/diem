address 0x1 {
module X {
    struct U {}
    resource struct R {}
}

module M {
    use 0x1::X::{U, U as U2};

    struct U {}
    struct U2 {}
    struct R2 {}
    struct R {}

    use 0x1::X::{R, R as R2};
}

}
