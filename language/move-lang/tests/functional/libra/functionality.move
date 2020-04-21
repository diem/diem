//! new-transaction
//! sender: association
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
// check: EXECUTED
