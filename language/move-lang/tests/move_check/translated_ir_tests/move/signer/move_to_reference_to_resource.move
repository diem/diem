module M {
    resource struct R { f: bool }
    fun t0(s: &signer, r: &R) {
        move_to<R>(s, move r)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

//! new-transaction

module N {
    resource struct R<T> { f: T }
    fun t0(s: &signer, r: &mut R<bool>) {
        move_to<R<bool>>(s, move r)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
