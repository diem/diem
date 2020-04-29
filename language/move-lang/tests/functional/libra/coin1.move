//! new-transaction
//! sender: association
script {
use 0x0::Libra;
use 0x0::Coin1;
use 0x0::Transaction;
use 0x0::FixedPoint32;
fun main() {
    Transaction::assert(Libra::approx_lbr_for_value<Coin1::T>(10) == 5, 1);
    Transaction::assert(Libra::scaling_factor<Coin1::T>() == 1000000, 2);
    Transaction::assert(Libra::fractional_part<Coin1::T>() == 100, 3);
    Libra::update_lbr_exchange_rate<Coin1::T>(FixedPoint32::create_from_rational(1, 3));
    Transaction::assert(Libra::approx_lbr_for_value<Coin1::T>(10) == 3, 4);
}
}
// check: EXECUTED
