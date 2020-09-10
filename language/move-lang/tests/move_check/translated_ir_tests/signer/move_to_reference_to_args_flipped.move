module M {
    resource struct R { f: bool }
    fun t0(s: &signer) {
        move_to<R>(R { f: false }, s)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

//! new-transaction

module N {
    resource struct R<T> { f: T }
    fun t0<T>(s: &signer) {
        move_to<R<bool>>(R { f: false }, s);
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
