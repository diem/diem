//! account: alice, 0, 0, address

//! new-transaction
//! sender: blessed
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice", false
stdlib_script::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
//! execute-as: alice
script {
use 0x1::AccountLimits;
use 0x1::Coin1;
use 0x1::Signer;
fun main(lr_account: &signer, vasp: &signer) {
    AccountLimits::publish_unrestricted_limits<Coin1::Coin1>(vasp);
    AccountLimits::publish_window<Coin1::Coin1>(
        lr_account,
        vasp,
        Signer::address_of(vasp)
    );
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: testnetdd
//! type-args: 0x1::Coin1::Coin1
//! args: {{alice}}, 1000000, b"", b""
stdlib_script::peer_to_peer_with_metadata
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::Coin1;

fun main(account: &signer) {
    AccountLimits::update_limits_definition<Coin1::Coin1>(
        account,
        {{alice}},
        0,
        100,
        0,
        0,
    );
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
//! gas-price: 150
//! gas-currency: Coin1
//! max-gas: 700
script {
    fun main() {
        abort 0
    }
}
// check: "Keep(ABORTED { code: 0, location: Script })"
