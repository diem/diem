module 0x1::A {
    struct A has key { }

    #[test]
    fun x() {
        abort 0
    }

    #[test(a=@0x1)]
    fun y(a: signer) {
        move_to(&a, A {});
        abort 0
    }

    // make sure that we only show storage state for failed tests

    #[test]
    #[expected_failure(abort_code = 0)]
    fun a() {
        abort 0
    }

    #[test(a=@0x1)]
    #[expected_failure(abort_code = 0)]
    fun b(a: signer) {
        move_to(&a, A {});
        abort 0
    }

    #[test(a=@0x1)]
    fun c(a: signer) {
        move_to(&a, A {});
    }
}
