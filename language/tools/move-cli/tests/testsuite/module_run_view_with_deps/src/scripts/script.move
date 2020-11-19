script {
    use 0x2::Test;
    use 0x1::Debug;

    fun main(account1: &signer, account2: &signer, x: u64, y: u64) {
        Debug::print(&x);
        Test::publish(account1, x);
        Debug::print(&y);
        Test::publish(account2, y)
    }
}
