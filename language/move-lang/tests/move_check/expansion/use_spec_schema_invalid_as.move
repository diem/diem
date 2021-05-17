address 0x2 {
module X {
    spec schema Foo<T> {
        ensures true;
    }
}

module M {
    use 0x2::X::{Foo as foo};
    struct S {}

    spec t {
        apply foo<S> to t;
    }
}

}
