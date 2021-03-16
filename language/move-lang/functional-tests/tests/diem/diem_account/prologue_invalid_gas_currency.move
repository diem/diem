//! account: alice, 0, 0, address

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice", false
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
//! execute-as: alice
script {
use 0x1::AccountLimits;
use 0x1::XUS;
use 0x1::Signer;
fun main(dr_account: signer, vasp: signer) {
    let dr_account = &dr_account;
    let vasp = &vasp;
    AccountLimits::publish_unrestricted_limits<XUS::XUS>(vasp);
    AccountLimits::publish_window<XUS::XUS>(
        dr_account,
        vasp,
        Signer::address_of(vasp)
    );
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: testnetdd
//! type-args: 0x1::XUS::XUS
//! args: {{alice}}, 1000000, b"", b""
stdlib_script::PaymentScripts::peer_to_peer_with_metadata
// check: "Keep(EXECUTED)"

// XXX/FIXME: invalid gas currency for account if it doesn't hold it is bad
//! new-transaction
//! sender: alice
//! gas-price: 1
//! max-gas: 1000
//! gas-currency: XDX
script {
    fun main() {
    }
}
// XXX/FIXME
// check: "Discard(BAD_TRANSACTION_FEE_CURRENCY)"
