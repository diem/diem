// The fees collected is less than the number of validators

//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator
//! sender: vivian
//! gas-price: 0
script {
use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Transaction;

fun main() {
  let number_of_validators = LibraSystem::validator_set_size();
  Transaction::assert(number_of_validators > 1, 0);

  let lib_coin = LibraAccount::withdraw_from_sender<LBR::T>(number_of_validators - 1);
  LibraAccount::deposit<LBR::T>(0xFEE, lib_coin);
}
}
//! check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: vivian
script {
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::LibraSystem;
use 0x0::Transaction;

fun main() {
  let number_of_validators = LibraSystem::validator_set_size();
  Transaction::assert(LibraAccount::balance<LBR::T>(0xFEE) == number_of_validators - 1, 3);
}
}
//! check: EXECUTED
