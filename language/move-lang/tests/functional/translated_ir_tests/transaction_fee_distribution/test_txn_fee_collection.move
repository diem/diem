//! account: alice, 90000

//! sender: alice
//! gas-price: 1

use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::TransactionFeeAccounts;
use 0x0::Transaction;

fun main() {
    let x = 10;
    // Make sure that the pot is empty to start with
    let fee_addr = TransactionFeeAccounts::transaction_fee_address<LBR::T>();
    let transaction_fee_pot_amount = LibraAccount::balance<LBR::T>(fee_addr);
    Transaction::assert(transaction_fee_pot_amount == 0, 0);
    // Do some work
    while (x > 0) {
        x = x - 1;
    }
}
//! check: EXECUTED

//! new-transaction
//! sender: alice
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::TransactionFeeAccounts;
use 0x0::Transaction;

fun main() {
    let previous_balance = 90000;
    let current_balance = LibraAccount::balance<LBR::T>({{alice}});
    Transaction::assert(previous_balance > current_balance, 1);
    // Calculate the transaction fee paid
    let transaction_fee_paid = previous_balance - current_balance;
    let fee_addr = TransactionFeeAccounts::transaction_fee_address<LBR::T>();
    let transaction_fee_pot_amount = LibraAccount::balance<LBR::T>(move fee_addr);
    // Assert that the transaction fees collected are equal to the transaction fees paid
    Transaction::assert(transaction_fee_paid == transaction_fee_pot_amount, 2);
}
//! check: EXECUTED
