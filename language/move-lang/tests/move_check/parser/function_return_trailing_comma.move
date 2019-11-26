module M {
    // Trailing commas are *not* allowed in multi-value types.
    f(): (u64, u64,) { (1, 2) }
}
