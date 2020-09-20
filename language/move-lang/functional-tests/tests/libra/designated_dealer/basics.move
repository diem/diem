//! account: bob, 0,0, address
//! account: validatorvivian, 10000000Coin1, 0, validator

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::publish_designated_dealer_credential<Coin1>(
        account, account, false
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::publish_designated_dealer_credential<Coin1>(
        account, account, false
    );
}
}
// check: "Keep(ABORTED { code: 1539,"

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::add_currency<Coin1>(account, account);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::add_currency<Coin1>(account, account);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::add_tier<Coin1>(account, {{bob}}, 0);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{bob}}, {{bob::auth_key}}, x"", false

stdlib_script::create_designated_dealer
// check: CreateAccountEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::add_tier<Coin1>(account, {{bob}}, 1000000000000);
}
}
// check: "Keep(ABORTED { code: 263,"

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    DesignatedDealer::update_tier<Coin1>(account, {{bob}}, 10, 1000000000000);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    DesignatedDealer::update_tier<Coin1>(account, {{bob}}, 0, 500000 * Libra::scaling_factor<Coin1>());
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    DesignatedDealer::update_tier<Coin1>(account, {{bob}}, 0, 5000000 * Libra::scaling_factor<Coin1>());
}
}
// check: "Keep(ABORTED { code: 519,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    DesignatedDealer::update_tier<Coin1>(account, {{bob}}, 2, 5000000 * Libra::scaling_factor<Coin1>());
}
}
// check: "Keep(ABORTED { code: 519,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::Coin1::Coin1;
use 0x1::Libra;
fun main(account: &signer) {
    Libra::destroy_zero(
        DesignatedDealer::tiered_mint<Coin1>(account, 0, {{bob}}, 0)
    );
}
}
// check: "Keep(ABORTED { code: 1031,"

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Libra;
    fun main(tc_account: &signer) {
        LibraAccount::tiered_mint<Coin1>(
            tc_account, {{bob}},  500000 * Libra::scaling_factor<Coin1>() - 1, 0
        );
    }
}
// check: ReceivedMintEvent
// check: MintEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(tc_account: &signer) {
        LibraAccount::tiered_mint<Coin1>(
            tc_account, {{bob}},  2, 0
        );
    }
}
// check: "Keep(ABORTED { code: 1287,"

//! block-prologue
//! proposer: validatorvivian
//! block-time: 95000000000

//! new-transaction
//! sender: blessed
//! expiration-time: 95000000001
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Libra;
    fun main(tc_account: &signer) {
        LibraAccount::tiered_mint<Coin1>(
            tc_account, {{bob}},  500000 * Libra::scaling_factor<Coin1>() - 1, 0
        );
    }
}
// check: ReceivedMintEvent
// check: MintEvent
// check: "Keep(EXECUTED)"
