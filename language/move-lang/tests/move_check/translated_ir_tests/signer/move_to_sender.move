module M {
    resource struct R { s: signer }
    fun t(s1: &signer, s: signer) {
        move_to(s1, R { s })
    }
}
