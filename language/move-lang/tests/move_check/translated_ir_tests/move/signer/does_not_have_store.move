module 0x8675309::M {
    struct S<T> has key {}

    fun t(_account: &signer) {
        move_to(account, S<signer> {})
    }
}
