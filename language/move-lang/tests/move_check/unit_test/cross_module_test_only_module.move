// check that modules that are annotated as test_only are filtered out correctly
address 0x1 {
#[test_only]
module M {
    public fun foo() { }
}

module Tests {
    // this use should cause an unbound module error as M should be filtered out
    use 0x1::M;

    fun bar() {
        M::foo()
    }
}
}
