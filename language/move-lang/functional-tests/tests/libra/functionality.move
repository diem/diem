//! account: bob, 0Coin1

module BurnCapabilityHolder {
    use 0x1::Libra;
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
use 0x1::Libra;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::Offer;
fun main(account: &signer) {
    let coin1 = Libra::mint<Coin1>(account, 10000);
    let coin2 = Libra::mint<Coin2>(account, 10000);
    assert(Libra::value<Coin1>(&coin1) == 10000, 0);
    assert(Libra::value<Coin2>(&coin2) == 10000, 1);
    assert(Libra::value<Coin2>(&coin2) == 10000, 1);

    let (coin11, coin12) = Libra::split(coin1, 5000);
    let (coin21, coin22) = Libra::split(coin2, 5000);
    assert(Libra::value<Coin1>(&coin11) == 5000 , 0);
    assert(Libra::value<Coin2>(&coin21) == 5000 , 1);
    assert(Libra::value<Coin1>(&coin12) == 5000 , 2);
    assert(Libra::value<Coin2>(&coin22) == 5000 , 3);
    let tmp = Libra::withdraw(&mut coin11, 1000);
    assert(Libra::value<Coin1>(&coin11) == 4000 , 4);
    assert(Libra::value<Coin1>(&tmp) == 1000 , 5);
    Libra::deposit(&mut coin11, tmp);
    assert(Libra::value<Coin1>(&coin11) == 5000 , 6);
    let coin1 = Libra::join(coin11, coin12);
    let coin2 = Libra::join(coin21, coin22);
    assert(Libra::value<Coin1>(&coin1) == 10000, 7);
    assert(Libra::value<Coin2>(&coin2) == 10000, 8);
    Offer::create(account, coin1, {{blessed}});
    Offer::create(account, coin2, {{blessed}});

    Libra::destroy_zero(Libra::zero<Coin1>());
    Libra::destroy_zero(Libra::zero<Coin2>());
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin1::Coin1;
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
    use 0x1::Libra;
    use 0x1::Coin1::Coin1;
    fun main()  {
        let coins = Libra::zero<Coin1>();
        Libra::approx_lbr_for_coin<Coin1>(&coins);
        Libra::destroy_zero(coins);
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x1::Libra;
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
    use 0x1::Libra;
    use 0x1::LBR::LBR;
    use 0x1::Coin1::Coin1;
    fun main()  {
        assert(!Libra::is_synthetic_currency<Coin1>(), 9);
        assert(Libra::is_synthetic_currency<LBR>(), 10);
        assert(!Libra::is_synthetic_currency<u64>(), 11);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
script {
    use 0x1::Libra;
    use 0x1::LibraTimestamp;
    fun main(account: &signer)  {
        LibraTimestamp::reset_time_has_started_for_test();
        Libra::initialize(account);
    }
}
// check: CANNOT_WRITE_EXISTING_RESOURCE

//! new-transaction
//! sender: blessed
script {
    use 0x1::Libra;
    use 0x1::Coin1::Coin1;
    use {{default}}::BurnCapabilityHolder;
    fun main(account: &signer)  {
        BurnCapabilityHolder::hold(
            account,
            Libra::remove_burn_capability<Coin1>(account)
        );
    }
}
// check: EXECUTED
