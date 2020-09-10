address 0x2 {
module X {
    struct S {}
    public fun foo() {}
}

module M {
    use 0x2::X::{Self as B, foo, S};

    struct X { f: X::S, f2: S }
    fun bar() {
        X::foo();
        foo()
    }
}
}
