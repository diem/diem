module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &address) {
        move_to(s, R { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0<T>(s: address) {
        move_to(s, R { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
