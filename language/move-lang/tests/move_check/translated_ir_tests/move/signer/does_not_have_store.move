module M {
    struct S<T> has key {}

    fun t(_account: &signer) {
        move_to(account, S<signer> {})
    }
}
