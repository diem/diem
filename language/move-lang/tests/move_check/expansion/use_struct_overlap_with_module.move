address 0x1 {
module X {
    struct S {}
}

module M {
    use 0x1::X::{Self, S as X};
    struct A { f1: X, f2: X::S }
}
}
