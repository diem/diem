//! account: bob, 100000000Coin1, 0, unhosted
//! account: alice, 0Coin1, 0, unhosted

//! new-transaction
//! sender: blessed
script {
    use 0x0::Coin1;
    use 0x0::LibraAccount;
    fun main(account: &signer) {
        LibraAccount::mint_to_address<Coin1::T>(account, {{bob}}, 10001);
    }
}
// TODO: fix account limits
// chec: ABORTED
// chec: 9

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    fun main(account: &signer) {
        LibraAccount::deposit(
            account,
            {{bob}},
            LibraAccount::withdraw_from<Coin1::T>(account, 10001)
        );
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
