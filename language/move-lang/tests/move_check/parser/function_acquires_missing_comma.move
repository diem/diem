module M {
    resource struct X1 {}
    resource struct X2 {}
    // Test a missing comma in the acquires list.
    f() acquires X1 X2 {
        borrow_global_mut<X1>(0x1);
        borrow_global_mut<X2>(0x1);
    }
}
