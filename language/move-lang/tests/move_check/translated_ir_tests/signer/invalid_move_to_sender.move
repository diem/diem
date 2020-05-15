module M {
    fun t(s: signer) {
        move_to_sender<signer>(move s)
    }
}

module N {
    resource struct R { s: signer }
    fun t(s: signer) {
        move_to_sender<R>(move s);
    }
}
