// dep: ../stdlib/modules/transaction.move
address 0x0:

module Txn {
    use 0x0::Transaction;

    resource struct T {
        value: u128,
    }

    fun check_sender1() {
        Transaction::assert(Transaction::sender() == 0xdeadbeef, 1);
    }
    spec fun check_sender1 {
        aborts_if sender() != 0xdeadbeef;
    }


    fun check_sender2() acquires T {
	borrow_global<T>(Transaction::sender());
    }
    spec fun check_sender2 {
        aborts_if !exists<T>(sender());
    }
}
