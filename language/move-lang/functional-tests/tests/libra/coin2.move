//! new-transaction
//! sender: blessed
script {
use 0x1::Libra;
use 0x1::Coin2::Coin2;
use 0x1::FixedPoint32;
fun main(account: &signer) {
    assert(Libra::approx_lbr_for_value<Coin2>(10) == 5, 1);
    assert(Libra::scaling_factor<Coin2>() == 1000000, 2);
    assert(Libra::fractional_part<Coin2>() == 100, 3);
    Libra::update_lbr_exchange_rate<Coin2>(account, FixedPoint32::create_from_rational(1,3));
    assert(Libra::approx_lbr_for_value<Coin2>(10) == 3, 4);
}
}
// check: EXECUTED
