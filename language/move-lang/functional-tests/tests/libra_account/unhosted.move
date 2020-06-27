//! account: bob, 100000000Coin1, 0, unhosted
//! account: alice, 0Coin1, 0, unhosted

//! new-transaction
//! sender: blessed
script {
    use 0x1::Coin1::Coin1;
    use 0x1::Libra;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let coins = Libra::mint<Coin1>(account, 10001);
        LibraAccount::deposit(account, {{bob}}, coins);
    }
}
// TODO: fix account limits
// chec: ABORTED
// chec: 9

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::deposit(
            account,
            {{bob}},
            LibraAccount::withdraw_from<Coin1>(&with_cap, 10001)
        );
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix account limits
// chec: ABORTED
// chec: 11

//! new-transaction
//! sender: bob
//! gas-price: 1000
//! gas-currency: Coin1
script {
    fun main() { }
}
// TODO: fix account limits

// chec: ABORTED
// chec: 11
