module 0x8675309::M {
    struct S<T> has key { f: T }

    fun t(account: &signer, s: signer) {
        move_to(account, S<signer> { f: s })
    }
}
