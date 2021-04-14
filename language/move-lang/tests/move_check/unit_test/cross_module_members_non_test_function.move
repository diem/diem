// check that `use`'s are filtered out correctly in non-test mode
address 0x1 {
module A {
    struct Foo has drop {}

    public fun build_foo(): Foo { Foo {} }
}

module B {
    #[test_only]
    use 0x1::A::{Self, Foo};

    #[test_only]
    fun x(_: Foo) { }

    #[test]
    fun tester() {
        x(A::build_foo())
    }

    // this should fail find the A module in non-test mode as the use statement
    // for `A` is test_only.
    public fun bad(): Foo {
        A::build_foo()
    }
}
}
