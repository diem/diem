//! account: alice, 90000
//! account: bob, 90000

//! sender: alice
//! args: {{bob}}
use 0x0::Transaction;
use 0x0::LibraAccount;

fun main(receiver: address) {
    let balance_before = LibraAccount::balance(Transaction::sender());
    LibraAccount::deposit(receiver, LibraAccount::withdraw_from_sender(200));
    let balance_after = LibraAccount::balance(Transaction::sender());
    Transaction::assert(balance_before == balance_after + 200, 42);
}
// check: EXECUTED


//! new-transaction
//! sender: bob
use 0x0::Transaction;
use 0x0::LibraAccount;

fun main() {
    Transaction::assert(LibraAccount::balance(Transaction::sender()) == 90200, 42);
}
// check: EXECUTED


//! new-transaction
//! sender: bob
//! args: {{alice}}
use 0x0::LibraAccount;

fun main(receiver: address) {
    LibraAccount::deposit(receiver, LibraAccount::withdraw_from_sender(100000));
}
// check: ABORTED 10
