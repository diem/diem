address 0x1 {

module X {
    struct S {}
    public fun foo() {}
}

module Y {
    struct S {}
    public fun foo() {}
}

module Z {
    struct S {}
    public fun foo() {}
}

module M {

    struct A { f1: XS, f2: S, f3: Z}

    use 0x1::X::{S as XS, foo};
    use 0x1::Z::{
        S as Z,
        foo as zfoo,
    };

    public fun m() {
        foo();
        Foo();
        zfoo();
    }

    use 0x1::Y::S;
    use 0x1::Y::foo as Foo;
}
}
