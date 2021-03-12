module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &signer) {
        move_to<R>();
    }
}
// check: NEGATIVE_STACK_SIZE_WITHIN_BLOCK

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0<T>(s: &signer) {
        () = move_to<R<bool>>();
    }
}
// check: NEGATIVE_STACK_SIZE_WITHIN_BLOCK
