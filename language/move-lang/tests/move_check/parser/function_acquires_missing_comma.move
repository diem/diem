module M {
    struct X1 {}
    struct X2 {}
    // Test a missing comma in the acquires list.
    fun f() acquires X1 X2 {
        borrow_global_mut<X1>(0x1);
        borrow_global_mut<X2>(0x1);
    }
}
