//! account: alice
//! account: bob
//! account: carol

//! sender: alice
module SillyColdWallet {
    use 0x1::XUS::XUS;
    use 0x1::DiemAccount;
    use 0x1::Diem;
    use 0x1::Signer;

    resource struct T {
        cap: DiemAccount::WithdrawCapability,
        owner: address,
    }

    public fun publish(account: &signer, cap: DiemAccount::WithdrawCapability, owner: address) {
        move_to(account, T { cap, owner });
    }

    public fun withdraw(account: &signer, wallet_address: address, _amount: u64): Diem::Diem<XUS> acquires T {
        let wallet_ref = borrow_global_mut<T>(wallet_address);
        let sender = Signer::address_of(account);
        assert(wallet_ref.owner == sender, 77);
        // TODO: the withdraw_from API is no longer exposed in DiemAccount
        Diem::zero()
    }
}

//! new-transaction
//! sender: alice
script {
use {{alice}}::SillyColdWallet;
use 0x1::DiemAccount;

// create a cold wallet for Bob that withdraws from Alice's account
fun main(sender: &signer) {
    let cap = DiemAccount::extract_withdraw_capability(sender);
    SillyColdWallet::publish(sender, cap, {{bob}});
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;

// check that Alice can no longer withdraw from her account
fun main(account: &signer) {
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    // should fail with withdrawal capability already extracted
    DiemAccount::pay_from<XUS>(&with_cap, {{alice}}, 1000, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 1793,"

//! new-transaction
//! sender: bob
script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;

// check that Bob can still withdraw from his normal account
fun main(account: &signer) {
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, {{bob}}, 1000, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}

//! new-transaction
//! sender: carol
script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;

// check that other users can still pay into Alice's account in the normal way
fun main(account: &signer) {
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, {{alice}}, 1000, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
