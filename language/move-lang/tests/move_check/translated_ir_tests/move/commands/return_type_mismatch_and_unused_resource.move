module 0x8675309::M {
    struct X {}

    fun t1(): bool {
        let x = X {};
        &x;
        false
    }

    fun t2(): &u64 {
        let u = 0;
        let r = &u;
        r
    }
}

// check: RET_UNSAFE_TO_DESTROY_ERROR
// check: RET_UNSAFE_TO_DESTROY_ERROR
