module A {
    use 0x0::Transaction;
    resource struct T1 {v: u64}
    resource struct T2 {v: u64}

    public fun test1() acquires T1, T2 {
        let x = borrow_global_mut<T1>(Transaction::sender());
        acquires_t2();
        acquires_t1();
        move x;
    }

    public fun test2() acquires T1, T2 {
        let x = borrow_global_mut<T1>(Transaction::sender());
        acquires_t2();
        acquires_t1();
        move x;
    }

    public fun test3() acquires T1, T2 {
        let x = borrow_global_mut<T1>(Transaction::sender());
        acquires_t1();
        move x;
        acquires_t2();
    }

    fun acquires_t1() acquires T1 {
        T1 { v: _ } = move_from<T1>(Transaction::sender());
    }

    fun acquires_t2() acquires T2 {
        T2 { v: _ } = move_from<T2>(Transaction::sender());
    }

}
