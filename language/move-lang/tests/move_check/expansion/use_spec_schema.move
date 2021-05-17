address 0x2 {
module X {
    spec schema Foo<T> {
        ensures true;
    }

    spec schema Bar<T> {
        ensures true;
    }
}

module M {
    use 0x2::X::{Foo, Bar as Baz};
    struct S {}
    fun t() {
    }

    spec t {
        apply Foo<S> to t;
        apply Baz<S> to t;
    }
}

}
