// #[test_only] structs should be filtered out in non-test mode
address 0x1 {
module M {
    #[test_only]
    struct Foo {}

    // This should cause an unbound type error in non-test mode as the Foo
    // struct declaration was filtered out
    public fun foo(): Foo {
        Foo {}
    }
}
}
