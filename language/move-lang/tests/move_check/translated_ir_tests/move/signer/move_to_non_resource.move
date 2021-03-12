module 0x8675309::M {
    struct R { f: bool }
    fun t0(s: &signer) {
        move_to(s, R { f: false })
    }
}
// check: MOVETO_NO_RESOURCE_ERROR

//! new-transaction

module 0x8675309::N {
    struct R<T> { f: T }
    fun t0<T: store>(s: &signer) {
        () = move_to(s, R { f: false })
    }
}
// check: MOVETO_NO_RESOURCE_ERROR
