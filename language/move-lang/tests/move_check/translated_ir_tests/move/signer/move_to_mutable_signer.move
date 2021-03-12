module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &mut signer) {
        // implicit freeze
        move_to(s, R { f: false })
    }
}

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0(s: &mut signer) {
        // implicit freeze
        () = move_to(s, R<bool> { f: false })
    }
}
