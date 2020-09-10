module M {
    resource struct R { f: bool }
    fun t0(s: &signer) {
        move_to<R>();
    }
}
// check: NEGATIVE_STACK_SIZE_WITHIN_BLOCK

module N {
    resource struct R<T> { f: T }
    fun t0<T>(s: &signer) {
        () = move_to<R<bool>>();
    }
}
// check: NEGATIVE_STACK_SIZE_WITHIN_BLOCK
