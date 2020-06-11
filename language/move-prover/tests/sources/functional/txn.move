address 0x1 {

module TestTransaction {
    use 0x1::Transaction;

    spec module {
        pragma verify = true;
    }


    resource struct T {
        value: u128,
    }

    fun check_sender1() {
        assert(Transaction::sender() == 0xdeadbeef, 1);
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

}
