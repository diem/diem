//! account: bob, 0Coin1
//! account: c1, 0Coin1
//! account: c2, 0Coin2

module BurnCapabilityHolder {
    use 0x0::Libra;
    resource struct Holder<Token> {
        cap: Libra::BurnCapability<Token>,
    }

    public fun hold<Token>(account: &signer, cap: Libra::BurnCapability<Token>) {
        move_to(account, Holder<Token>{ cap })
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::Libra;
use 0x0::LibraAccount;
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::Transaction;
fun main(account: &signer) {
    let coin1 = Libra::mint<Coin1>(account, 10000);
    let coin2 = Libra::mint<Coin2>(account, 10000);
    Transaction::assert(Libra::value<Coin1>(&coin1) == 10000, 0);
    Transaction::assert(Libra::value<Coin2>(&coin2) == 10000, 1);
    Transaction::assert(Libra::value<Coin2>(&coin2) == 10000, 1);

    let (coin11, coin12) = Libra::split(coin1, 5000);
    let (coin21, coin22) = Libra::split(coin2, 5000);
    Transaction::assert(Libra::value<Coin1>(&coin11) == 5000 , 0);
    Transaction::assert(Libra::value<Coin2>(&coin21) == 5000 , 1);
    Transaction::assert(Libra::value<Coin1>(&coin12) == 5000 , 2);
    Transaction::assert(Libra::value<Coin2>(&coin22) == 5000 , 3);
    let tmp = Libra::withdraw(&mut coin11, 1000);
    Transaction::assert(Libra::value<Coin1>(&coin11) == 4000 , 4);
    Transaction::assert(Libra::value<Coin1>(&tmp) == 1000 , 5);
    Libra::deposit(&mut coin11, tmp);
    Transaction::assert(Libra::value<Coin1>(&coin11) == 5000 , 6);
    let coin1 = Libra::join(coin11, coin12);
    let coin2 = Libra::join(coin21, coin22);
    Transaction::assert(Libra::value<Coin1>(&coin1) == 10000, 7);
    Transaction::assert(Libra::value<Coin2>(&coin2) == 10000, 8);
    LibraAccount::deposit(account, {{c1}}, coin1);
    LibraAccount::deposit(account, {{c2}}, coin2);

    Libra::destroy_zero(Libra::zero<Coin1>());
    Libra::destroy_zero(Libra::zero<Coin2>());
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x0::Libra;
use 0x0::Coin1::Coin1;
fun main(account: &signer) {
    Libra::destroy_zero(Libra::mint<Coin1>(account, 1));
}
}
// check: ABORTED
// check: 5

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
    use 0x0::Libra;
    use 0x0::Coin1::Coin1;
    fun main()  {
        let coins = Libra::zero<Coin1>();
        Libra::approx_lbr_for_coin<Coin1>(&coins);
        Libra::destroy_zero(coins);
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x0::Libra;
    fun main()  {
        Libra::destroy_zero(
            Libra::zero<u64>()
        );
    }
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x0::Libra;
    use 0x0::LBR::LBR;
    use 0x0::Coin1::Coin1;
    use 0x0::Transaction;
    fun main()  {
        Transaction::assert(!Libra::is_synthetic_currency<Coin1>(), 9);
        Transaction::assert(Libra::is_synthetic_currency<LBR>(), 10);
        Transaction::assert(!Libra::is_synthetic_currency<u64>(), 11);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::Libra;
    fun main(account: &signer)  {
        Libra::initialize(account);
    }
}
// check: ABORTED
// check: 0

//! new-transaction
//! sender: blessed
script {
use 0x0::LibraAccount;
use 0x0::Coin1::Coin1;
fun main(account: &signer)  {
    LibraAccount::mint_to_address<Coin1>(account, {{bob}}, 1000000000 * 1000000 + 1);
}
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: blessed
script {
    use 0x0::Libra;
    use 0x0::Coin1::Coin1;
    fun main(account: &signer)  {
        Libra::publish_mint_capability(
            account,
            Libra::remove_mint_capability<Coin1>(account)
        );
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x0::Libra;
    use 0x0::Coin1::Coin1;
    use {{default}}::BurnCapabilityHolder;
    fun main(account: &signer)  {
        BurnCapabilityHolder::hold(
            account,
            Libra::remove_burn_capability<Coin1>(account)
        );
    }
}
// check: EXECUTED
