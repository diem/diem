module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &signer, r: R) {
        move_to(s, r);
        move_to(s, R { f: false })
    }
}

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0<T: store>(s: &signer, r: R<T>) {
        move_to(s, r);
        move_to(s, R { f: false })
    }
}
