script {
    use 0x1::Debug;
    fun main<T: copyable>(_account1: &signer, _account2: &signer, b: bool, u: u64) {
        // TODO(tmn) cleanup after abilities migration. Currently function is not backwards
        // compatible because of signer
        // Debug::print(account1);
        // Debug::print(account2);
        Debug::print(&b);
        Debug::print(&u);
    }
}
