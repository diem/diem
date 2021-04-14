// #[test_only] functions should be filtered out in non-test mode
address 0x1 {
module M {
    // This should cause an unbound function error in non-test mode as `bar`
    // was filtered out
    public fun foo() { bar() }

    #[test_only]
    public fun bar() { }
}
}
