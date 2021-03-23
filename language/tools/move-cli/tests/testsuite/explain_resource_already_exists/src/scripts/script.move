script {
    use 0x2::ResourceExists;
    fun main(account: signer) {
        ResourceExists::f(&account);
    }
}
