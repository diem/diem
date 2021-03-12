module 0x8675309::A {
    struct T1 has key {v: u64}

    public fun test() acquires T1 {
    }

}

// check: EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR
