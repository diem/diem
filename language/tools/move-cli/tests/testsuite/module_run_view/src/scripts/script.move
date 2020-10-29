script {
    use 0x2::Test;
    fun main(account1: &signer, account2: &signer, x: u64, y: u64) {
        Test::publish(account1, x);
        Test::publish(account2, y)
    }
}
