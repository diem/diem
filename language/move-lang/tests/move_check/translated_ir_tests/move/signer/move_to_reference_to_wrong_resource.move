module 0x8675309::M {
    struct R has key { f: bool }
    struct X has key { f: bool }
    fun t0(s: &signer) {
        move_to<R>(s, X { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

//! new-transaction

module 0x8675309::N {
    struct R<T> has key { f: T }
    struct X<T> has key { f: T }
    fun t0(s: &signer) {
        () = move_to<X<bool>>(s, R { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
