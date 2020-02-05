module A {
    use 0x0::Transaction;
    resource struct T {v: u64}

    public fun t0(cond: bool) acquires T {
        let sender = Transaction::sender();
        let x = borrow_global_mut<T>(sender);
        copy x;
        x = borrow_global_mut<T>(sender);
        copy x;
    }
}

// check: GLOBAL_REFERENCE_ERROR
