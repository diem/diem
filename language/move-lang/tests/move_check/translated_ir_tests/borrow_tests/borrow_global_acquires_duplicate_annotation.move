module A {
    use 0x0::Transaction;
    resource struct T1{v: u64}

    public fun test() acquires T1, T1 {
        borrow_global_mut<T1>(Transaction::sender());
    }
}

// check: DUPLICATE_ACQUIRES_RESOURCE_ANNOTATION_ERROR
