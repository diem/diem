script {
    use 0x2::Events;

    fun emit(account: signer, i: u64) {
        Events::emit(&account, i)
    }
}
