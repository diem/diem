//! account: alice
script {
    use 0x1::RegisteredCurrencies;
    fun main(account: &signer) {
        RegisteredCurrencies::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::RegisteredCurrencies;
    fun main(account: &signer) {
        RegisteredCurrencies::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::RegisteredCurrencies;
    fun main(account: &signer) {
        RegisteredCurrencies::add_currency_code(account, b"XDX");
    }
}
// check: "Keep(ABORTED { code: 7,"
