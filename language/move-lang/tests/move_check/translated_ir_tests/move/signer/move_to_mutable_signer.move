module M {
    struct R has key { f: bool }
    fun t0(s: &mut signer) {
        // implicit freeze
        move_to(s, R { f: false })
    }
}

module N {
    struct R<T> has key { f: T }
    fun t0(s: &mut signer) {
        // implicit freeze
        () = move_to(s, R<bool> { f: false })
    }
}
