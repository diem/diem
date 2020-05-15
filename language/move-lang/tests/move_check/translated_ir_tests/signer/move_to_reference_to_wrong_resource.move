module M {
    resource struct R { f: bool }
    resource struct X { f: bool }
    fun t0(s: &signer) {
        move_to<R>(s, X { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

//! new-transaction

module N {
    resource struct R<T> { f: T }
    resource struct X<T> { f: T }
    fun t0(s: &signer) {
        () = move_to<X<bool>>(s, R { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
