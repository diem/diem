module 0x8675309::M {
    // Trailing commas are *not* allowed in multi-value types.
    fun f(): (u64, u64,) { (1, 2) }
}
