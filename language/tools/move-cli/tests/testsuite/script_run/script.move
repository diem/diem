script {
    use 0x1::Debug;
    fun main<T>(account1: &signer, account2: &signer, b: bool, u: u64) {
        Debug::print(account1);
        Debug::print(account2);
        Debug::print(&b);
        Debug::print(&u);
    }
}
