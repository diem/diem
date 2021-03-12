module 0x8675309::M {
    fun t0(s: &signer) {
        move_to<u64>(s, 0)
    }
}
// check: ParserError: Invalid Token

//! new-transaction

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0<T>(s: &signer) {
        move_to<R<u64>>(s, 0)
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
