// Check that test_only filtering and calling is supported across modules and
// different types of module members
address 0x1 {
module A {
    #[test_only]
    struct Foo has drop {}

    #[test_only]
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
}
}
