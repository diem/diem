module A {
    resource struct T1{ b: bool }

    public fun test(addr: address) {
        borrow_global<T1>(addr);
    }

}

// check: MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR
