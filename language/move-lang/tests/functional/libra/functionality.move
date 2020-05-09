//! account: bob, 0Coin1

module BurnCapabilityHolder {
    use 0x0::Libra;
    resource struct Holder<Token> {
        cap: Libra::BurnCapability<Token>,
    }

    public fun hold<Token>(account: &signer, cap: Libra::BurnCapability<Token>) {
        move_to(account, Holder<Token>{ cap })
    }
}

//! new-transaction
//! sender: blessed
script {
use 0x0::Libra;
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::Transaction;
fun main() {
    let pre_coin1 = Libra::new_preburn<Coin1::T>();
    let pre_coin2 = Libra::new_preburn<Coin2::T>();
    Libra::publish_preburn(pre_coin1);
    Libra::publish_preburn(pre_coin2);

    let coin1 = Libra::mint<Coin1::T>(10000);
    let coin2 = Libra::mint<Coin2::T>(10000);
    Transaction::assert(Libra::value<Coin1::T>(&coin1) == 10000, 0);
    Transaction::assert(Libra::value<Coin2::T>(&coin2) == 10000, 1);
    Transaction::assert(Libra::value<Coin2::T>(&coin2) == 10000, 1);

    let (coin11, coin12) = Libra::split(coin1, 5000);
    let (coin21, coin22) = Libra::split(coin2, 5000);
    Transaction::assert(Libra::value<Coin1::T>(&coin11) == 5000 , 0);
    Transaction::assert(Libra::value<Coin2::T>(&coin21) == 5000 , 1);
    Transaction::assert(Libra::value<Coin1::T>(&coin12) == 5000 , 2);
    Transaction::assert(Libra::value<Coin2::T>(&coin22) == 5000 , 3);
    let tmp = Libra::withdraw(&mut coin11, 1000);
    Transaction::assert(Libra::value<Coin1::T>(&coin11) == 4000 , 4);
    Transaction::assert(Libra::value<Coin1::T>(&tmp) == 1000 , 5);
    Libra::deposit(&mut coin11, tmp);
    Transaction::assert(Libra::value<Coin1::T>(&coin11) == 5000 , 6);
    let coin1 = Libra::join(coin11, coin12);
    let coin2 = Libra::join(coin21, coin22);
    Transaction::assert(Libra::value<Coin1::T>(&coin1) == 10000, 7);
    Transaction::assert(Libra::value<Coin2::T>(&coin2) == 10000, 8);

    Libra::preburn_to_sender(coin1);
    Libra::preburn_to_sender(coin2);
    Libra::burn<Coin1::T>(Transaction::sender());
    Libra::burn<Coin2::T>(Transaction::sender());
    Libra::destroy_zero(Libra::zero<Coin1::T>());
    Libra::destroy_zero(Libra::zero<Coin2::T>());
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x0::Libra;
    use 0x0::Coin1;
    fun main()  {
        Libra::destroy_zero(Libra::mint<Coin1::T>(1));
    }
}
// check: ABORTED
// check: 5

//! new-transaction
//! sender: bob
//! gas-currency: Coin1
script {
    use 0x0::Libra;
    use 0x0::Coin1;
    fun main()  {
        let coins = Libra::zero<Coin1::T>();
        Libra::approx_lbr_for_coin<Coin1::T>(&coins);
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
    use 0x0::LBR;
    use 0x0::Coin1;
    use 0x0::Transaction;
    fun main()  {
        Transaction::assert(!Libra::is_synthetic_currency<Coin1::T>(), 9);
        Transaction::assert(Libra::is_synthetic_currency<LBR::T>(), 10);
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
    use 0x0::Coin1;
    fun main()  {
        LibraAccount::mint_to_address<Coin1::T>({{bob}}, 1000000000 * 1000000 + 1);
    }
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: blessed
script {
    use 0x0::Libra;
    use 0x0::Coin1;
    fun main(account:  &signer)  {
        Libra::publish_mint_capability(
            account,
            Libra::remove_mint_capability<Coin1::T>()
        );
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x0::Libra;
    use 0x0::Coin1;
    use {{default}}::BurnCapabilityHolder;
    fun main(account: &signer)  {
        BurnCapabilityHolder::hold(
            account,
            Libra::remove_burn_capability<Coin1::T>()
        );
    }
}
// check: EXECUTED
