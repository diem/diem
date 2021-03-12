module 0x8675309::A {
    fun three(): (u64, u64, u64) {
        (0, 1, 2)
    }

    fun pop() {
        (_, _) = three();
    }
}

// check: POSITIVE_STACK_SIZE_AT_BLOCK_END
