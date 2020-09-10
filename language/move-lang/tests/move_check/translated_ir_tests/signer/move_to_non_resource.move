module M {
    struct R { f: bool }
    fun t0(s: &signer) {
        move_to(s, R { f: false })
    }
}
// check: MOVETO_NO_RESOURCE_ERROR

//! new-transaction

module N {
    struct R<T> { f: T }
    fun t0<T>(s: &signer) {
        () = move_to(s, R { f: false })
    }
}
// check: MOVETO_NO_RESOURCE_ERROR
