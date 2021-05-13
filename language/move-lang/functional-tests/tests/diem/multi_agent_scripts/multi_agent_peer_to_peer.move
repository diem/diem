//! new-transaction
//! account: alice
//! account: bob
//! sender: alice
//! secondary-signers: bob
//! args: 10
script {
use DiemFramework::DiemAccount;
use Std::Signer;
use DiemFramework::XUS;

fun main(alice: signer, bob: signer, amount: u64) {
    let alice_withdrawal_cap = DiemAccount::extract_withdraw_capability(&alice);
    let bob_addr = Signer::address_of(&bob);
    DiemAccount::pay_from<XUS::XUS>(
        &alice_withdrawal_cap, bob_addr, amount, x"", x""
    );
    DiemAccount::restore_withdraw_capability(alice_withdrawal_cap);
}
}
