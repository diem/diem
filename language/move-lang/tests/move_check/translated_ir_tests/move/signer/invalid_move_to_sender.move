module M {
    fun t(s1: &signer, s: signer) {
        move_to<signer>(s1, move s)
    }
}

module N {
    struct R has key { s: signer }
    fun t(s1: &signer, s: signer) {
        move_to<R>(s1, move s);
    }
}
