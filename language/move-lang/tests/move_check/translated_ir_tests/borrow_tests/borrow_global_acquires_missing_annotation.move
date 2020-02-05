module A {
    use 0x0::Transaction;
    resource struct T1{v: u64}

    public fun test() {
        borrow_global_mut<T1>(Transaction::sender());
    }

}

// check: MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR
