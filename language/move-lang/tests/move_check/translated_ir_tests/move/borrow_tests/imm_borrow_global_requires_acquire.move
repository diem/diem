module 0x8675309::A {
    struct T1 has key { b: bool }

    public fun test(addr: address) {
        borrow_global<T1>(addr);
    }

}

// check: MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR
