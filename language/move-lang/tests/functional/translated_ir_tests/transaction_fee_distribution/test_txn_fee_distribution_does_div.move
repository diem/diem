// Number of validators evenly divides the transaction fees collected

//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator
//! account: vanessa, 1000000, 0, validator
//! sender: vivian
//! gas-price: 0
module BalanceHolder {
    use 0x0::Vector;
    use 0x0::Transaction;

    resource struct T {
        balances: vector<u64>,
    }

    public fun publish(balances: vector<u64>) {
        move_to_sender<T>(T { balances: balances })
    }

    public fun get_balance(i: u64): u64 acquires T {
        *Vector::borrow<u64>(&borrow_global<T>(Transaction::sender()).balances, i)
    }
}

//! new-transaction
//! sender: alice
//! gas-price: 0

use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Vector;
use 0x0::TransactionFeeAccounts;
use {{vivian}}::BalanceHolder;

fun main() {
  let index = 0;
  let balances = Vector::empty<u64>();
  let number_of_validators = LibraSystem::validator_set_size();
  // Withdraw now so that the new account balance for {{vivian}}'s account is recorded in the
  // balances vector.
  let lib_coin = LibraAccount::withdraw_from_sender<LBR::T>(number_of_validators * 10);

  // Make the distribution check agnostic to the starting balances of the validators
  while (index < number_of_validators) {
      let addr = LibraSystem::get_ith_validator_address(index);
      index = index + 1;
      Vector::push_back<u64>(
          &mut balances,
          LibraAccount::balance<LBR::T>(addr)
      );
  };

  BalanceHolder::publish(balances);
  let fee_addr = TransactionFeeAccounts::transaction_fee_address<LBR::T>();
  LibraAccount::deposit<LBR::T>(fee_addr, lib_coin);
}
//! check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: alice
use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::TransactionFeeAccounts;
use 0x0::Transaction;
use {{vivian}}::BalanceHolder;

fun main() {
  let index = 0;
  let number_of_validators = LibraSystem::validator_set_size();

  while (index < number_of_validators) {
     let addr = LibraSystem::get_ith_validator_address(index);
     let new_balance = LibraAccount::balance<LBR::T>(addr);
     let old_balance = BalanceHolder::get_balance(index);
     index = index + 1;
     Transaction::assert(new_balance == (old_balance + 10), 77);
  };

  let fee_addr = TransactionFeeAccounts::transaction_fee_address<LBR::T>();
  Transaction::assert(LibraAccount::balance<LBR::T>(fee_addr) == 0, 10000);
}
//! check: EXECUTED
