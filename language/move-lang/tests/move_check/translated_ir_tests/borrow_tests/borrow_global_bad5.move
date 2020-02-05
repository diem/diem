module A {
    use 0x0::Transaction;
    resource struct T {v: u64}

    public fun A5(b: bool) acquires T {
        let t = T { v: 0 };
        let t1_ref = if (b) borrow_global_mut<T>(Transaction::sender()) else &mut t;
        let t2_ref = borrow_global_mut<T>(Transaction::sender());
        t1_ref;
        t2_ref;
        T { v: _ } = t;
    }
}

// check: GLOBAL_REFERENCE_ERROR
