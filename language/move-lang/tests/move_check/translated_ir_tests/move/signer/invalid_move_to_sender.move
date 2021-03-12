module 0x8675309::M {
    fun t(s1: &signer, s: signer) {
        move_to<signer>(s1, move s)
    }
}

module 0x8675309::N {
    struct R has key { s: signer }
    fun t(s1: &signer, s: signer) {
        move_to<R>(s1, move s);
    }
}
