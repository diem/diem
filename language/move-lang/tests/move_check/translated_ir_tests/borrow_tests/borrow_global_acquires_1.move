module A {
    use 0x0::Transaction;
    resource struct T1 {v: u64}

    public fun test(addr: address) acquires T1 {
        borrow_global_mut<T1>(Transaction::sender());
        acquires_t1();
    }

    fun acquires_t1() acquires T1 {
        T1 { v: _ } = move_from<T1>(Transaction::sender());
    }

}
