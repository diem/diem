module 0x8675309::M {
    struct X has key {}
    // Test a trailing comma in the acquires list.
    fun f() acquires X, {
        borrow_global_mut<X>(@0x1);
    }
}
