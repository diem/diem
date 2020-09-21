module M {
    resource struct R { f: bool }
    fun t0(s: &mut signer) {
        // implicit freeze
        move_to(s, R { f: false })
    }
}

module N {
    resource struct R<T> { f: T }
    fun t0(s: &mut signer) {
        // implicit freeze
        () = move_to(s, R<bool> { f: false })
    }
}
