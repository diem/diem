address 0x1 {
module X {
    spec schema Foo<T> {
        ensures true;
    }
}

module M {
    use 0x1::X::{Foo as foo};
    struct S {}

    spec fun t {
        apply foo<S> to t;
    }
}

}
