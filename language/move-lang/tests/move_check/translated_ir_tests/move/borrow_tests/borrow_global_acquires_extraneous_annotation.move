module A {
    resource struct T1 {v: u64}

    public fun test() acquires T1 {
    }

}

// check: EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR
