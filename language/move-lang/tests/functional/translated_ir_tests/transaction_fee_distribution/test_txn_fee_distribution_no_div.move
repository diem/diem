// Number of validators doesn't evenly divide the transaction fees collected

//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator

//! sender: vivian
module BalanceHolder {
    use 0x0::Vector;

    resource struct T {
        balances: vector<u64>,
    }

    public fun publish(balances: vector<u64>) {
        move_to_sender<T>(T { balances: balances })
    }

    public fun get_balance(i: u64): u64 acquires T {
        *Vector::borrow<u64>(&borrow_global<T>(0x0::Transaction::sender()).balances, i)
    }
}

//! new-transaction
//! sender: alice
//! gas-price: 0

script {
use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Vector;
use 0x0::Transaction;
use {{vivian}}::BalanceHolder;

fun main() {
  let index = 0;
  let balances = Vector::empty<u64>();
  let number_of_validators = LibraSystem::validator_set_size();

  // Withdraw now so that the new account balance for {{vivian}}'s account is recorded in the
  // balances vector. Withdraw a number that does not evenly divide the number of validators, this
  // also means that we need more than 1 validator in the validator set.
  Transaction::assert(number_of_validators > 1, 0);
  let lib_coin = LibraAccount::withdraw_from_sender<LBR::T>(number_of_validators * 10 + 1);

  // Make the distribution check agnostic to the starting balances of the validators
  while (index < number_of_validators) {
      let addr = LibraSystem::get_ith_validator_address(index);
      index = index + 1;
      Vector::push_back<u64>(
          &mut balances,
          LibraAccount::balance<LBR::T>(addr)
      );
  };

  LibraAccount::deposit<LBR::T>(0xFEE, lib_coin);
  BalanceHolder::publish(balances);
}
}
//! check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: alice
script {
use 0x0::LibraSystem;
use 0x0::LibraAccount;
use 0x0::LBR;
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

  // The remainder in the transaction fee balance must be less than the number of validators.
  Transaction::assert(LibraAccount::balance<LBR::T>(0xFEE) < number_of_validators, 2);
  // And in this particular case, must be exactly equal to 1.
  Transaction::assert(LibraAccount::balance<LBR::T>(0xFEE) == 1, 3);
}
}
//! check: EXECUTED
