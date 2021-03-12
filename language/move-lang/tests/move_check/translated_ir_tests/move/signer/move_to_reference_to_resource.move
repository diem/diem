module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &signer, r: &R) {
        move_to<R>(s, move r)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

//! new-transaction

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0(s: &signer, r: &mut R<bool>) {
        move_to<R<bool>>(s, move r)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
