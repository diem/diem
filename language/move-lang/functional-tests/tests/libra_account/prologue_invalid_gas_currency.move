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

// XXX/FIXME: invalid gas currency for account if it doesn't hold it is bad
//! new-transaction
//! sender: alice
//! gas-price: 1
//! max-gas: 1000
script {
    fun main() {
    }
}
// XXX/FIXME
// check: "Discard(INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE)"
