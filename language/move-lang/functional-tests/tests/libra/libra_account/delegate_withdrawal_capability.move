//! account: alice
//! account: bob
//! account: carol

//! sender: alice
module SillyColdWallet {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    use 0x1::Libra;
    use 0x1::Signer;

    resource struct T {
        cap: LibraAccount::WithdrawCapability,
        owner: address,
    }

    public fun publish(account: &signer, cap: LibraAccount::WithdrawCapability, owner: address) {
        move_to(account, T { cap, owner });
    }

    public fun withdraw(account: &signer, wallet_address: address, _amount: u64): Libra::Libra<Coin1> acquires T {
        let wallet_ref = borrow_global_mut<T>(wallet_address);
        let sender = Signer::address_of(account);
        assert(wallet_ref.owner == sender, 77);
        // TODO: the withdraw_from API is no longer exposed in LibraAccount
        Libra::zero()
    }
}

//! new-transaction
//! sender: alice
script {
use {{alice}}::SillyColdWallet;
use 0x1::LibraAccount;

// create a cold wallet for Bob that withdraws from Alice's account
fun main(sender: &signer) {
    let cap = LibraAccount::extract_withdraw_capability(sender);
    SillyColdWallet::publish(sender, cap, {{bob}});
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

// check that Alice can no longer withdraw from her account
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    // should fail with withdrawal capability already extracted
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1000, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 1793,"

//! new-transaction
//! sender: bob
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

// check that Bob can still withdraw from his normal account
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 1000, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}

//! new-transaction
//! sender: carol
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;

// check that other users can still pay into Alice's account in the normal way
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 1000, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
