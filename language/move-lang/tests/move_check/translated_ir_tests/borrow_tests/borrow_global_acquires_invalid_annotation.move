module A {
    resource struct T1 {v: u64}

    public fun test() acquires T1 {
        test()
    }

}

// check: INVALID_ACQUIRES_RESOURCE_ANNOTATION_ERROR
