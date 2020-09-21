module M {
    resource struct R { f: bool }
    fun t0(s: &address) {
        move_to(s, R { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR

module N {
    resource struct R<T> { f: T }
    fun t0<T>(s: address) {
        move_to(s, R { f: false })
    }
}
// check: MOVETO_TYPE_MISMATCH_ERROR
