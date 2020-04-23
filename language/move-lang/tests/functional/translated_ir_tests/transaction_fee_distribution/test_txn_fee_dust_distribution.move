// The fees collected is less than the number of validators

//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator
//! sender: vivian
//! gas-price: 0

use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Transaction;
use 0x0::TransactionFeeAccounts;

fun main() {
  let number_of_validators = LibraSystem::validator_set_size();
  Transaction::assert(number_of_validators > 1, 0);

  let lib_coin = LibraAccount::withdraw_from_sender<LBR::T>(number_of_validators - 1);
  let fee_addr = TransactionFeeAccounts::transaction_fee_address<LBR::T>();
  LibraAccount::deposit<LBR::T>(fee_addr, lib_coin);
}
//! check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: vivian
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::LibraSystem;
use 0x0::TransactionFeeAccounts;
use 0x0::Transaction;

fun main() {
  let number_of_validators = LibraSystem::validator_set_size();
  let fee_addr = TransactionFeeAccounts::transaction_fee_address<LBR::T>();
  Transaction::assert(LibraAccount::balance<LBR::T>(fee_addr) == number_of_validators - 1, 3);
}
//! check: EXECUTED
