module A {
    use 0x0::Transaction;
    resource struct T1 {v: u64}

    public fun test1() acquires T1 {
        borrow_acquires_t1();
    }

    fun borrow_acquires_t1(): &mut T1 acquires T1 {
        borrow_global_mut<T1>(Transaction::sender())
    }
}

// check: RET_UNSAFE_TO_DESTROY_ERROR
