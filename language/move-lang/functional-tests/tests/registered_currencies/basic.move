//! account: alice
script {
    use 0x1::RegisteredCurrencies;
    fun main(account: &signer) {
        RegisteredCurrencies::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::RegisteredCurrencies;
    fun main(account: &signer) {
        RegisteredCurrencies::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::RegisteredCurrencies;
    fun main(account: &signer) {
        RegisteredCurrencies::add_currency_code(account, b"LBR");
    }
}
// check: "Keep(ABORTED { code: 2,"
