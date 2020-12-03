//! account: bob, 0,0, address
//! account: validatorvivian, 10000000XUS, 0, validator

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DesignatedDealer::publish_designated_dealer_credential<XUS>(
        account, account, false
    );
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DesignatedDealer::publish_designated_dealer_credential<XUS>(
        account, account, false
    );
}
}
// check: "Keep(ABORTED { code: 1539,"

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DesignatedDealer::add_currency<XUS>(account, account);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DesignatedDealer::add_currency<XUS>(account, account);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{bob}}, {{bob::auth_key}}, x"", false

stdlib_script::create_designated_dealer
// check: CreateAccountEvent
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
fun main(account: &signer) {
    DesignatedDealer::update_tier<XUS>(account, {{bob}}, 10, 1000000000000);
}
}
// check: "Keep(ABORTED { code: 258,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
use 0x1::Diem;
fun main(account: &signer) {
    DesignatedDealer::update_tier<XUS>(account, {{bob}}, 0, 500000 * Diem::scaling_factor<XUS>());
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
use 0x1::Diem;
fun main(account: &signer) {
    DesignatedDealer::update_tier<XUS>(account, {{bob}}, 0, 5000000 * Diem::scaling_factor<XUS>());
}
}
// check: "Keep(ABORTED { code: 519,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
use 0x1::Diem;
fun main(account: &signer) {
    DesignatedDealer::update_tier<XUS>(account, {{bob}}, 2, 5000000 * Diem::scaling_factor<XUS>());
}
}
// check: "Keep(ABORTED { code: 519,"

//! new-transaction
//! sender: blessed
script {
use 0x1::DesignatedDealer;
use 0x1::XUS::XUS;
use 0x1::Diem;
fun main(account: &signer) {
    Diem::destroy_zero(
        DesignatedDealer::tiered_mint<XUS>(account, 0, {{bob}}, 0)
    );
}
}
// check: "Keep(ABORTED { code: 1031,"

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    use 0x1::Diem;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, {{bob}},  500000 * Diem::scaling_factor<XUS>() - 1, 0
        );
    }
}
// check: ReceivedMintEvent
// check: MintEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
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
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    use 0x1::Diem;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, {{bob}},  500000 * Diem::scaling_factor<XUS>() - 1, 0
        );
    }
}
// check: ReceivedMintEvent
// check: MintEvent
// check: "Keep(EXECUTED)"
