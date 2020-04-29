
//! new-transaction
//! sender: association
script {
use 0x0::Libra;
use 0x0::LBR;
use 0x0::Transaction;
fun main() {
    Transaction::assert(Libra::approx_lbr_for_value<LBR::T>(10) == 10, 1);
    Transaction::assert(Libra::scaling_factor<LBR::T>() == 1000000, 2);
    Transaction::assert(Libra::fractional_part<LBR::T>() == 1000, 3);
}
}
// check: EXECUTED
