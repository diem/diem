// Make sure that the `SentPaymentEvent` for transaction fee distribution
// shows 0xFEE as the prefix of the sender address.

//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator
//! sender: vivian

script {
use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Transaction;

fun main() {
  let number_of_validators = LibraSystem::validator_set_size();

  Transaction::assert(number_of_validators > 1, 0);

  let lib_coin = LibraAccount::withdraw_from_sender<LBR::T>(number_of_validators);
  LibraAccount::deposit<LBR::T>(0xFEE, lib_coin);
}
}
//! check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

// check: SentPaymentEvent
// check: 0fee
