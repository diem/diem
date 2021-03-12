module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &signer) {
        move_to<R>(R { f: false }, s)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

//! new-transaction

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0<T>(s: &signer) {
        move_to<R<bool>>(R { f: false }, s);
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
