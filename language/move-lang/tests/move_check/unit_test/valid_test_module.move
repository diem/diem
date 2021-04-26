// Make sure that legal usage is allowed
module 0x1::M {
    // test-only struct
    #[test_only]
    struct Foo {}

    public fun foo() { }

    // test-only struct used in a test-only function
    #[test_only]
    public fun bar(): Foo { Foo{} }

    // tests with no arguments are allowed
    #[test]
    public fun go() { }

    // proper number of signers, and correct parameter names
    #[test(_a=@0x1)]
    public fun a(_a: signer) { }

    // multiple signers are supported
    #[test(_a=0x1, _b=@0x2)]
    public fun b(_a: signer, _b: signer) { }

    // multiple attributes are supported in the same annotation
    #[test(_a=@0x1, _b=@0x2), expected_failure]
    public fun c(_a: signer, _b: signer) { }

    // Can also break them apart into separate annotations
    #[test(_a=@0x1, _b=@0x2)]
    #[expected_failure]
    public fun d(_a: signer, _b: signer) { }

    // Can assign abort codes
    #[test(_a=@0x1, _b=@0x2)]
    #[expected_failure(abort_code=5)]
    public fun e(_a: signer, _b: signer) { }

    // singe annotation with multiple attributes and an abort code assignment
    #[test(_a=@0x1, _b=@0x2), expected_failure(abort_code=5)]
    public fun g(_a: signer, _b: signer) { }

    // single annotation without any arguments and an abort code assignment
    #[test, expected_failure(abort_code=5)]
    public fun h() { }

    // single annotation with no arguments and no abort code annotation
    #[test, expected_failure]
    public fun i() { }
}
