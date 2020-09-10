module M {
    fun t(s1: &signer, s: signer) {
        move_to<signer>(s1, move s)
    }
}

module N {
    resource struct R { s: signer }
    fun t(s1: &signer, s: signer) {
        move_to<R>(s1, move s);
    }
}
