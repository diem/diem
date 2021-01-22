module M {
    resource struct S<T> {}

    fun t(_account: &signer) {
        // TODO(tmn) update when abilities hit
        // move_to(account, S<signer> {})
    }
}
