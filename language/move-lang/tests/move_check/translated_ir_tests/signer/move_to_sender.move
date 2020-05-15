module M {
    resource struct R { s: signer }
    fun t(s: signer) {
        move_to_sender(R { s })
    }
}
