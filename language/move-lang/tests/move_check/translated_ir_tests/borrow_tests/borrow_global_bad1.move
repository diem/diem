module A {
    use 0x0::Transaction;
    resource struct T {v: u64}

    public fun A0(addr: address) acquires T {
        let x = borrow_global_mut<T>(Transaction::sender());
        let y = borrow_global_mut<T>(addr);
        x;
        y;
    }
}

// check: GLOBAL_REFERENCE_ERROR
