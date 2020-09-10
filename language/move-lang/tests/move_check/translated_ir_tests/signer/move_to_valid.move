module M {
    resource struct R { f: bool }
    fun t0(s: &signer, r: R) {
        move_to(s, r);
        move_to(s, R { f: false })
    }
}

module N {
    resource struct R<T> { f: T }
    fun t0<T>(s: &signer, r: R<T>) {
        move_to(s, r);
        move_to(s, R { f: false })
    }
}
